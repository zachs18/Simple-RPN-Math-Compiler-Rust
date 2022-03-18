use crate::{compiler::{Variable, Location, CompileError}, code::{Relocatable, Object, Relocation, RelocationKind, Symbol}};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Machine;

macro_rules! make_register_type {
    (
        $caller_saved:ident;
        $callee_saved:ident;
        $unusable:ident;
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $value:literal $usability:ident),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant = $value),*
        }
        const ALL_REGISTERS: &[$name] = &[ $($name::$variant),* ];
        impl $name {
            fn is_caller_saved(&self) -> bool {
                let $caller_saved = true;
                let $callee_saved = false;
                let $unusable = false;
                use $name::*;
                match self {
                    $(
                        $variant => $usability,
                    )*
                }
            }
            fn is_callee_saved(&self) -> bool {
                let $caller_saved = false;
                let $callee_saved = true;
                let $unusable = false;
                use $name::*;
                match self {
                    $(
                        $variant => $usability,
                    )*
                }
            }
            fn is_unusable(&self) -> bool {
                let $caller_saved = false;
                let $callee_saved = false;
                let $unusable = true;
                use $name::*;
                match self {
                    $(
                        $variant => $usability,
                    )*
                }
            }
        }
    }
}

make_register_type!{
caller_saved;
callee_saved;
unusable;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum Register {
    Rax = 0 caller_saved,
    Rcx = 1 caller_saved,
    Rdx = 2 caller_saved,
    Rbx = 3 callee_saved,
    Rsp = 4 unusable,
    Rbp = 5 unusable,
    Rsi = 6 caller_saved,
    Rdi = 7 caller_saved,
    R8 = 8 caller_saved,
    R9 = 9 caller_saved,
    R10 = 10 caller_saved,
    R11 = 11 caller_saved,
    R12 = 12 callee_saved,
    R13 = 13 callee_saved,
    R14 = 14 callee_saved,
    R15 = 15 unusable,
    Rip = 16 unusable,
}
}

struct RegisterDisplacement {
    register: Register,
    disp: i32
}

macro_rules! impl_from_for_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident($fieldty:ty)),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant($fieldty)),*
        }
        $(
        impl From<$fieldty> for $name {
            fn from(field: $fieldty) -> Self {
                $name::$variant(field)
            }
        }
        )*
    }
}

impl_from_for_enum!{
enum Operand {
    Register(Register),
    RegisterDisplacement(RegisterDisplacement),
    Symbol(Symbol), // Rip-relative
}
}

macro_rules! make_simple_instruction_type {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $src_rm:tt / $dst_rm:tt),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant),*
        }
        impl $name {
            fn src_rm_opcode(self) -> Option<u8> {
                use $name::*;
                match self {
                    $(
                        $variant => $src_rm.into(),
                    )*
                }
            }
            fn dst_rm_opcode(self) -> Option<u8> {
                use $name::*;
                match self {
                    $(
                        $variant => $dst_rm.into(),
                    )*
                }
            }
        }
    }
}

make_simple_instruction_type!{
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SimpleInstruction {
    Mov = 0x8b / 0x89,
    Add = 0x03 / 0x01,
    Sub = 0x2b / 0x29,
    Lea = 0x8d / None,
}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum ConditionalJump {
    Overflow = 0x0,
    NoyOverflow = 0x1,
    Below = 0x2,
    AboveOrEqual = 0x3,
    Equal = 0x4,
    NotEqual = 0x5,
    BelowOrEqual = 0x6,
    Above = 0x7,
    Sign = 0x8,
    NotSign = 0x9,
    ParityEven = 0xa,
    ParityOdd = 0xb,
    Less = 0xc,
    GreaterOrEqual = 0xd,
    LessOrEqual = 0xe,
    Greater = 0xf,
}

impl ConditionalJump {
    fn short_jump(self, dst: Symbol) -> Relocatable {
        let bytes = vec![0x70 | self as u8, 0x00];
        Relocatable{
            data: bytes.into(),
            symbols: vec![],
            abs_symbols: vec![],
            relocations: vec![
                Relocation::new(1, RelocationKind::Pc8, dst, -1),
            ],
        }
    }
    fn near_jump(self, dst: Symbol) -> Relocatable {
        let bytes = vec![0x0f, 0x80 | self as u8, 0x00, 0x00, 0x00, 0x00];
        Relocatable{
            data: bytes.into(),
            symbols: vec![],
            abs_symbols: vec![],
            relocations: vec![
                Relocation::new(1, RelocationKind::Pc32, dst, -2),
            ],
        }
    }
}

impl Machine {
    fn simple(self, src: Operand, dst: Operand, instruction: SimpleInstruction) -> Result<Relocatable, ()> {
        use Operand::*;
        match (src, dst) {
            (Register(src), Register(dst)) => self.simple_reg_to_reg(src, dst, instruction),
            (Register(src), Symbol(dst)) => self.simple_reg_to_symbol(src, dst, instruction),
            (Symbol(src), Register(dst)) => self.simple_symbol_to_reg(src, dst, instruction),
            (Register(src), RegisterDisplacement(dst)) => self.simple_reg_to_memory(src, dst, instruction),
            (RegisterDisplacement(src), Register(dst)) => self.simple_memory_to_reg(src, dst, instruction),
            _ => todo!(),
        }
    }

    fn simple_reg_to_reg(self, src: Register, dst: Register, instruction: SimpleInstruction) -> Result<Relocatable, ()> {
        let src = src as u8;
        let dst = dst as u8;
        if src >= 16 || dst >= 16 { return Err(()); }
        let opcode = instruction.dst_rm_opcode().ok_or(())?;
        let mut bytes = [
            0x48, opcode, 0xc0,
        ];

        let rex_b = ((dst >> 3) & 0x1) << 0;
        let rex_r = ((src >> 3) & 0x1) << 2;
        bytes[0] |= rex_b | rex_r;

        let modrm_rm  = (dst & 0x7) << 0;
        let modrm_reg = (src & 0x7) << 3;
        bytes[2] |= modrm_rm | modrm_reg;
        Ok(bytes.into())
    }

    fn simple_reg_to_symbol(self, src: Register, dst: Symbol, instruction: SimpleInstruction) -> Result<Relocatable, ()> {
        let src = src as u8;
        if src >= 16 { return Err(()); }
        let opcode = instruction.dst_rm_opcode().ok_or(())?;
        let mut bytes = vec![
            0x48, opcode, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let rex_b = 0; // rip-relative uses b.r/m = 0.101
        let rex_r = ((src >> 3) & 0x1) << 2;
        bytes[0] |= rex_b | rex_r;

        let modrm_rm  = 0b101 << 0; // rip-relative uses b.r/m = 0.101
        let modrm_reg = (src & 0x7) << 3;
        bytes[2] |= modrm_rm | modrm_reg;
        Ok(Relocatable {
            data: bytes.into(),
            symbols: vec![],
            abs_symbols: vec![],
            relocations: vec![
                Relocation::new(3, RelocationKind::Pc32, dst, -4)
            ],
        })
    }

    fn simple_symbol_to_reg(self, src: Symbol, dst: Register, instruction: SimpleInstruction) -> Result<Relocatable, ()> {
        let dst = dst as u8;
        if dst >= 16 { return Err(()); }
        let opcode = instruction.src_rm_opcode().ok_or(())?;
        let mut bytes = vec![
            0x48, opcode, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let rex_b = 0; // rip-relative uses b.r/m = 0.101
        let rex_r = ((dst >> 3) & 0x1) << 2;
        bytes[0] |= rex_b | rex_r;

        let modrm_rm  = 0b101 << 0; // rip-relative uses b.r/m = 0.101
        let modrm_reg = (dst & 0x7) << 3;
        bytes[2] |= modrm_rm | modrm_reg;

        Ok(Relocatable {
            data: bytes.into(),
            symbols: vec![],
            abs_symbols: vec![],
            relocations: vec![
                Relocation::new(3, RelocationKind::Pc32, src, -4)
            ],
        })
    }

    fn simple_reg_to_memory(self, src: Register, dst: RegisterDisplacement, instruction: SimpleInstruction) -> Result<Relocatable, ()> {
        todo!() // https://wiki.osdev.org/X86-64_Instruction_Encoding#32.2F64-bit_addressing
    }

    fn simple_memory_to_reg(self, src: RegisterDisplacement, dst: Register, instruction: SimpleInstruction) -> Result<Relocatable, ()> {
        todo!() // https://wiki.osdev.org/X86-64_Instruction_Encoding#32.2F64-bit_addressing
    }
}


const ARG_REGISTERS: [Register; 6] = [
    Register::Rdi,
    Register::Rsi,
    Register::Rdx,
    Register::Rcx,
    Register::R8,
    Register::R9,
];

impl super::super::Machine for Machine {
    type Register = Register;
    type Clobber = Vec<Register>;

    fn function_prologue_epilogue_abort(self, stack_slots: usize, arg_slots: Vec<usize>) -> Result<(Object, Object, Object), CompileError<'static>> {
        for arg in &arg_slots {
            if *arg >= stack_slots {
                return Err(CompileError::Other(format!("invalid arg slot: {} is >= {}, the number of stack slots", arg, stack_slots).into()));
            }
        }
        let mut code: Vec<u8> = vec![
            0xf3, 0x0f, 0x1e, 0xfa,                     // endbr64
            0x55,                                       // push %rbp
            0x48, 0x89, 0xe5,                           // mov %rsp,%rbp
            0x48, 0x81, 0xec, 0x00, 0x00, 0x00, 0x00,   // sub $slots*8,%rsp
        ];
        let stack_slots = i32::try_from(stack_slots).unwrap();
        code[11..].copy_from_slice(&stack_slots.to_le_bytes());

        let mut arg_slots = arg_slots.into_iter();

        let mut code = Relocatable::from(code);

        for (reg, slot) in ARG_REGISTERS.into_iter().zip(arg_slots.by_ref().take(6)) {
            let mut bytes = [
                0x48, 0x89, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00,
            ];

            let reg = reg as u8;
    
            match reg >> 3 {
                0 => {},
                1 => {
                    bytes[0] |= 0x04;
                },
                _ => unreachable!(),
            };
            let mask = (reg & 0x07) << 3;
            bytes[2] |= mask;

            code += bytes.into();
        }

        for (frame_index, slot) in arg_slots.enumerate() {
            // Offset for the frame pointer and the return address
            let frame_index = frame_index + 2;
            let frame_offset = frame_index.checked_mul(8).unwrap();
            let disp = i32::try_from(frame_offset).unwrap();
            let src = RegisterDisplacement { register: Register::Rbp, disp };
            let stack_offset = slot.checked_mul(8).unwrap();
            let disp = i32::try_from(stack_offset).unwrap();
            let dst = RegisterDisplacement { register: Register::Rsp, disp };
            let intermediate = Register::Rax;

            code += self.simple(src.into(), intermediate.into(), SimpleInstruction::Mov).unwrap();
            code += self.simple(intermediate.into(), dst.into(), SimpleInstruction::Mov).unwrap();
        }

        todo!()
    }

    fn add_data(self, data: Vec<u8>, symbol: Symbol) -> Object {
        let data = Relocatable{
            data: data.into(),
            symbols: vec![(symbol, 0)],
            abs_symbols: vec![],
            relocations: vec![],
        };
        Object {
            code: Default::default(),
            data,
        }
    }

    fn usable_registers(self) -> Vec<Register> {
        ALL_REGISTERS.iter().copied().filter(Register::is_caller_saved).collect()
    }
    fn load_from(self, into: Register, from: &Variable) -> (Object, Self::Clobber) {
        let Variable { location, .. } = from;
        match location {
            Location::Local { stack_index } => {
                let stack_offset = stack_index.checked_mul(8).unwrap();
                let disp = i32::try_from(stack_offset).unwrap();
                let src = RegisterDisplacement { register: Register::Rsp, disp };
                
                (
                    self.simple_memory_to_reg(src, into, SimpleInstruction::Mov).unwrap().into(),
                    Default::default(),
                )
            },
            Location::Static { symbol, /*atomic*/ .. } => {
                // Aligned loads on amd64 are always atomic
                (
                    self.simple_symbol_to_reg(symbol.clone(), into, SimpleInstruction::Mov).unwrap().into(),
                    Default::default(),
                )
            },
        }
    }

    fn store_to(self, from: Register, into: &Variable) -> (Object, Self::Clobber) {
        let Variable { location, mutable, ..} = into;
        // if !mutable { panic!("TODO: error cannot change immutable variable"); } // NOTE: this doesn't work because we use store_to to initialize also
        match location {
            Location::Local { stack_index } => {
                let stack_offset = stack_index.checked_mul(8).unwrap();
                let disp = i32::try_from(stack_offset).unwrap();
                let dst = RegisterDisplacement { register: Register::Rsp, disp };
                
                (
                    self.simple_reg_to_memory(from, dst, SimpleInstruction::Mov).unwrap().into(),
                    Default::default(),
                )
            },
            Location::Static { symbol, /*atomic*/ .. } => {
                // Aligned stores on amd64 are always atomic
                (
                    self.simple_reg_to_symbol(from, symbol.clone(), SimpleInstruction::Mov).unwrap().into(),
                    Default::default(),
                )
            },
        }
    }

    fn copy_from(self, dst: Register, src: Register) -> (Object, Self::Clobber) {
        (
            self.simple_reg_to_reg(src, dst, SimpleInstruction::Mov).unwrap().into(),
            Default::default(),
        )
    }

    fn add_assign(self, dst: Register, src: Register) -> (Object, Self::Clobber) {
        (
            self.simple_reg_to_reg(src, dst, SimpleInstruction::Add).unwrap().into(),
            Default::default(),
        )
    }

    fn checked_add_assign(self, lhs: Register, rhs: Register) -> (Object, Self::Clobber) {
        todo!()
    }

    fn sub_assign(self, dst: Register, src: Register) -> (Object, Self::Clobber) {
        (
            self.simple_reg_to_reg(src, dst, SimpleInstruction::Sub).unwrap().into(),
            Default::default(),
        )
    }

    fn checked_sub_assign(self, lhs: Register, rhs: Register) -> (Object, Self::Clobber) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::Register::*;
    use crate::{compiler::Machine, code::Object};
    #[test]
    fn add_assign() {
        let machine = super::Machine;
        let (code, clobbers) = machine.add_assign(Rax, Rax);
        assert_eq!(code, Object::from([0x48, 0x01, 0xc0]));
        assert_eq!(clobbers, []);

        let (code, clobbers) = machine.add_assign(Rax, R8);
        assert_eq!(code, Object::from([0x4c, 0x01, 0xc0]));
        assert_eq!(clobbers, []);

        let (code, clobbers) = machine.add_assign(R15, Rax);
        assert_eq!(code, Object::from([0x49, 0x01, 0xc7]));
        assert_eq!(clobbers, []);

        let (code, clobbers) = machine.add_assign(R10, R10);
        assert_eq!(code, Object::from([0x4d, 0x01, 0xd2]));
        assert_eq!(clobbers, []);
    }
}