use crate::{compiler::{Variable, Location}, code::{Relocatable, Relocation, RelocationKind}};

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
}
}

impl super::super::Register for Register {
    type Clobber = Vec<Self>;
    fn usable_registers() -> Vec<Self> {
        ALL_REGISTERS.iter().copied().filter(Register::is_caller_saved).collect()
    }
    fn load_from(self, from: &Variable) -> (Relocatable, Self::Clobber) {
        let Variable {
            name,
            r#type,
            location,
        } = from;
        match location {
            Location::Local { stack_offset } => {
                let mut bytes = vec![0x48, 0x8b, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00u8];
                let register = self as u8;
                match register >> 3 {
                    0 => {},
                    1 => bytes[0] |= 0x04,
                    _ => unreachable!(),
                };
                let mask = (register & 0x07) << 3;
                bytes[2] |= mask;
                let stack_offset = i32::try_from(*stack_offset).unwrap();
                bytes[4..].copy_from_slice(&stack_offset.to_le_bytes());
                (
                    Relocatable::from(bytes),
                    Default::default(),
                )
            },
            Location::Static { symbol, /*atomic*/ .. } => {
                // Aligned loads on amd64 are always atomic
                let mut bytes = vec![0x48, 0x8b, 0x05, 0x00, 0x00, 0x00, 0x00u8];
                let register = self as u8;
                match register >> 3 {
                    0 => {},
                    1 => bytes[0] |= 0x04,
                    _ => unreachable!(),
                };
                let mask = (register & 0x07) << 3;
                bytes[2] |= mask;
                (
                    Relocatable {
                        data: bytes.into(),
                        symbols: vec![],
                        abs_symbols: vec![],
                        relocations: vec![
                            Relocation::new(
                                3,
                                RelocationKind::Pc32,
                                symbol.clone(),
                                -4
                            )
                        ],
                    },
                    Default::default(),
                )
            },
        }
    }

    fn store_to(self, into: &Variable) -> (Relocatable, Self::Clobber) {
        let Variable {
            name,
            r#type,
            location,
        } = into;
        match location {
            Location::Local { stack_offset } => {
                let mut bytes = vec![0x48, 0x89, 0x84, 0x24, 0x00, 0x00, 0x00, 0x00u8];
                let register = self as u8;
                match register >> 3 {
                    0 => {},
                    1 => bytes[0] |= 0x04,
                    _ => unreachable!(),
                };
                let mask = (register & 0x07) << 3;
                bytes[2] |= mask;
                let stack_offset = u32::try_from(*stack_offset).unwrap();
                bytes[4..].copy_from_slice(&stack_offset.to_le_bytes());
                (Relocatable {
                    data: bytes.into(),
                    symbols: vec![],
                    abs_symbols: vec![],
                    relocations: vec![],
                    // relocations: vec![
                    //     Relocation{ location: 4, kind: RelocationKind::32S, symbol: todo!(), addend: todo!() }
                    // ],
                }, Default::default())
            },
            Location::Static { symbol, /*atomic*/ .. } => {
                // Aligned stores on amd64 are always atomic
                let mut bytes = vec![0x48, 0x89, 0x05, 0x00, 0x00, 0x00, 0x00u8];
                let register = self as u8;
                match register >> 3 {
                    0 => {},
                    1 => bytes[0] |= 0x04,
                    _ => unreachable!(),
                };
                let mask = (register & 0x07) << 3;
                bytes[2] |= mask;
                (Relocatable {
                    data: bytes.into(),
                    symbols: vec![],
                    abs_symbols: vec![],
                    relocations: vec![
                        Relocation::new(
                            3,
                            RelocationKind::Pc32,
                            symbol.clone(),
                            -4
                        )
                    ],
                }, Default::default())
            },
        }
    }

    fn copy_from(self, src: Self) -> (Relocatable, Self::Clobber) {
        let mut bytes = vec![0x48, 0x89, 0xc0];
        let src_reg = src as u8;
        let dst_reg = self as u8;

        match src_reg >> 3 {
            0 => {},
            1 => {
                bytes[0] |= 0x01;
            },
            _ => unreachable!(),
        };
        let src_mask = (src_reg & 0x07) << 3;
        bytes[2] |= src_mask;

        match dst_reg >> 3 {
            0 => {},
            1 => {
                bytes[0] |= 0x04;
            },
            _ => unreachable!(),
        };
        let dst_mask = (dst_reg & 0x07) << 0;
        bytes[2] |= dst_mask;
        (
            Relocatable::from(bytes),
            Default::default(),
        )
    }

    fn add_assign(self, rhs: Self) -> (Relocatable, Self::Clobber) {
        let mut bytes = vec![0x48, 0x01, 0xc0];
        let dst_reg = self as u8;
        let src_reg = rhs as u8;

        match src_reg >> 3 {
            0 => {},
            1 => {
                bytes[0] |= 0x04;
            },
            _ => unreachable!(),
        };
        let src_mask = (src_reg & 0x07) << 3;
        bytes[2] |= src_mask;

        match dst_reg >> 3 {
            0 => {},
            1 => {
                bytes[0] |= 0x01;
            },
            _ => unreachable!(),
        };
        let dst_mask = (dst_reg & 0x07) << 0;
        bytes[2] |= dst_mask;
        (
            Relocatable::from(bytes),
            Default::default(),
        )
    }

    fn checked_add_assign(self, rhs: Self) -> (Relocatable, Self::Clobber) {
        todo!()
    }

    fn sub_assign(self, rhs: Self) -> (Relocatable, Self::Clobber) {
        let mut bytes = vec![0x48, 0x29, 0xc0];
        let dst_reg = self as u8;
        let src_reg = rhs as u8;

        match src_reg >> 3 {
            0 => {},
            1 => {
                bytes[0] |= 0x04;
            },
            _ => unreachable!(),
        };
        let src_mask = (src_reg & 0x07) << 3;
        bytes[2] |= src_mask;

        match dst_reg >> 3 {
            0 => {},
            1 => {
                bytes[0] |= 0x01;
            },
            _ => unreachable!(),
        };
        let dst_mask = (dst_reg & 0x07) << 0;
        bytes[2] |= dst_mask;
        (
            Relocatable::from(bytes),
            Default::default(),
        )
    }

    fn checked_sub_assign(self, rhs: Self) -> (Relocatable, Self::Clobber) {
        todo!()
    }
    
    
}

#[cfg(test)]
mod tests {
    use super::Register::*;
    use crate::{compiler::Register, code::Relocatable};
    #[test]
    fn add_assign() {
        let (code, clobbers) = Rax.add_assign(Rax);
        assert_eq!(code, Relocatable::from([0x48, 0x01, 0xc0]));
        assert_eq!(clobbers, []);

        let (code, clobbers) = Rax.add_assign(R8);
        assert_eq!(code, Relocatable::from([0x4c, 0x01, 0xc0]));
        assert_eq!(clobbers, []);

        let (code, clobbers) = R15.add_assign(Rax);
        assert_eq!(code, Relocatable::from([0x49, 0x01, 0xc7]));
        assert_eq!(clobbers, []);

        let (code, clobbers) = R10.add_assign(R10);
        assert_eq!(code, Relocatable::from([0x4d, 0x01, 0xd2]));
        assert_eq!(clobbers, []);
    }
}