use crate::commands::*;
use crate::raw_code::{function_header_code, function_footer_code};
use libc::{c_void, c_long, mmap, munmap, mprotect};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Function {
    // TODO: keep track of how many params it uses?
    code: *mut c_void,
    code_length: usize,
}

impl std::ops::Drop for Function {
    fn drop(&mut self) {
        let result = unsafe { munmap(self.code, self.code_length) };
        if result != 0 {
            todo!("handle munmap() failure");
        }
    }
}

impl Function {
    pub fn parse(mut s: &str) -> Result<Function, &'static str> {
        let (_param_count, commands) = Function::parse_helper(&mut s)?;
        s = s.trim_start();
        if s.len() != 0 {
            return Err("Unrecognized command");
        }
        // TODO: return param_count?
        Function::new(commands)
    }
    fn parse_uint(s: &mut &str) -> Result<usize, &'static str> {
        let mut value: usize;
        static DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        *s = match s.strip_prefix(DIGITS) {
            None => return Err("Not an integer"),
            Some(rest) => {
                value = s.chars().next().unwrap().to_digit(10).unwrap() as usize;
                rest
            },
        };
        while let Some(rest) = s.strip_prefix(DIGITS) {
            let digit = s.chars().next().unwrap().to_digit(10).unwrap() as usize;
            value = value
                .checked_mul(10)
                .ok_or("Integer literal too large")?
                .checked_add(digit)
                .ok_or("Integer literal too large")?;
            *s = rest;
        }
        Ok(value)
    }
    fn parse_iint(s: &mut &str) -> Result<isize, &'static str> {
        let negative: bool = match s.strip_prefix('-') {
            None => false,
            Some(rest) => { *s = rest; true }
        };
        let magnitude: usize = Function::parse_uint(s)?;
        if !negative {
            magnitude.try_into().map_err(|_| "Integer literal out of range")
        } else if magnitude <= isize::MAX as usize {
            Ok(-(magnitude as isize))
        } else if magnitude == isize::MIN as usize {
            Ok(magnitude as isize)
        } else {
            Err("Integer literal out of range")
        }
    }
    fn parse_helper(s: &mut &str) -> Result<(usize, Vec<Command>), &'static str> {
        let mut param_count = 0;
        let mut commands: Vec<Command> = vec![];
        while {*s = s.trim_start(); s.len() > 0} {
            // Commands are trimmed from s in their match
            match s.chars().next() {
                None => break,
                Some(next) => match next {
                    'a' => {
                        param_count = param_count.max(1);
                        commands.push(PUSH_A.clone());
                        *s = s.split_at(1).1;
                    },
                    'b' => {
                        param_count = param_count.max(2);
                        commands.push(PUSH_B.clone());
                        *s = s.split_at(1).1;
                    },
                    'c' => {
                        param_count = param_count.max(3);
                        commands.push(PUSH_C.clone());
                        *s = s.split_at(1).1;
                    },
                    'd' => {
                        param_count = param_count.max(4);
                        commands.push(PUSH_D.clone());
                        *s = s.split_at(1).1;
                    },
                    'e' => {
                        param_count = param_count.max(5);
                        commands.push(PUSH_E.clone());
                        *s = s.split_at(1).1;
                    },
                    'f' => {
                        param_count = param_count.max(6);
                        commands.push(PUSH_F.clone());
                        *s = s.split_at(1).1;
                    },
                    '+' => {
                        commands.push(ADD.clone());
                        *s = s.split_at(1).1;
                    },
                    '-' => {
                        commands.push(SUBTRACT.clone());
                        *s = s.split_at(1).1;
                    },
                    '*' => {
                        commands.push(MULTIPLY.clone());
                        *s = s.split_at(1).1;
                    },
                    '/' => {
                        commands.push(DIVIDE.clone());
                        *s = s.split_at(1).1;
                    },
                    '%' => {
                        commands.push(MOD.clone());
                        *s = s.split_at(1).1;
                    },
                    '\\' => {
                        commands.push(UDIVIDE.clone());
                        *s = s.split_at(1).1;
                    },
                    '@' => {
                        commands.push(UMOD.clone());
                        *s = s.split_at(1).1;
                    },
                    '0'..='9' => {
                        let value: isize =
                            Function::parse_uint(s)?
                            .try_into()
                            .map_err(|_| "Integer literal too large")?;
                        commands.push(PUSH_VALUE(value));
                    },
                    'l'|'p' => {
                        *s = s.split_at(1).1;
                        let index: i32 = 
                            Function::parse_iint(s)?
                            .try_into()
                            .map_err(|_| "Stack index out of range")?;
                        commands.push(PUSH_STACK_INDEX(index));
                    },
                    's' => {
                        *s = s.split_at(1).1;
                        let index: i32 = 
                            Function::parse_iint(s)?
                            .try_into()
                            .map_err(|_| "Stack index out of range")?;
                        commands.push(POP_STACK_INDEX(index));
                    },
                    '{' => {
                        *s = s.split_at(1).1;
                        let (loop_param_count, loop_commands) = Function::parse_helper(s)?;
                        *s = s.strip_prefix('}').ok_or("Loop ended without '}'")?;
                        param_count = param_count.max(loop_param_count);
                        commands.push(WHILE_LOOP(loop_commands)?);
                    },
                    '}' => break, // Caller should check that the &str is empty
                    _ => return Err("Unrecognized command"),
                },
            };
        }
        Ok((param_count, commands))
    }

    pub(crate) fn new(commands: Vec<Command>) -> Result<Function, &'static str> {
        let mut stack_size: usize = 0;
        let mut code = function_header_code().to_owned();

        for command in commands {
            if stack_size < command.param_count {
                return Err("Function would pop value from empty stack");
            }
            if stack_size < command.required_stack_depth {
                return Err("Function would use value from past end of stack");
            }
            stack_size -= command.param_count;
            stack_size += command.return_count;
            code.extend_from_slice(&*command.code);
        }
        if stack_size == 0 {
            return Err("Function would return from empty stack");
        }

        code.extend_from_slice(function_footer_code());

        let code_binary: *mut c_void = unsafe {
            mmap(
                std::ptr::null_mut(),
                code.len(),
                libc::PROT_READ|libc::PROT_WRITE|libc::PROT_EXEC,
                libc::MAP_PRIVATE|libc::MAP_ANONYMOUS,
                -1,
                0
            )
        };
        if code_binary.is_null() {
            return Err("mmap failed");
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                code.as_ptr(),
                code_binary as *mut u8,
                code.len(),
            )
        };

        if 0 != unsafe {mprotect(code_binary, code.len(), libc::PROT_READ|libc::PROT_EXEC)} {
            let result = unsafe { munmap(code_binary, code.len()) };
            if result != 0 {
                todo!("handle munmap() failure");
            }
            return Err("mprotect failed");
        }
        Ok(Function {
            code: code_binary,
            code_length: code.len(),
        })
    }
}

macro_rules! impl_unsafe_as_fn_ptr {
    ($name:ident, $args:tt) => {
        #[deny(unsafe_op_in_unsafe_fn)]
        pub unsafe fn $name(&self) -> extern "C" fn $args -> c_long {
            unsafe { std::mem::transmute(self.code) }
        }
    }
}
impl Function {
    impl_unsafe_as_fn_ptr!(as_fn_ptr_0, ());
    impl_unsafe_as_fn_ptr!(as_fn_ptr_1, (c_long));
    impl_unsafe_as_fn_ptr!(as_fn_ptr_2, (c_long, c_long));
    impl_unsafe_as_fn_ptr!(as_fn_ptr_3, (c_long, c_long, c_long));
    impl_unsafe_as_fn_ptr!(as_fn_ptr_4, (c_long, c_long, c_long, c_long));
    impl_unsafe_as_fn_ptr!(as_fn_ptr_5, (c_long, c_long, c_long, c_long, c_long));
    impl_unsafe_as_fn_ptr!(as_fn_ptr_6, (c_long, c_long, c_long, c_long, c_long, c_long));
}

macro_rules! impl_fn_traits {
    ($Args:ty, $as_fn_ptr:ident, $args:ident, ( $($args_expanded:tt)* ) ) => {
        #[cfg(feature = "fn_traits")]
        impl std::ops::FnOnce<$Args> for Function {
            type Output = c_long;
            extern "rust-call" fn call_once(self, args: $Args) -> Self::Output {
                self.call(args)
            }
        }

        #[cfg(feature = "fn_traits")]
        impl std::ops::FnMut<$Args> for Function {
            extern "rust-call" fn call_mut(&mut self, args: $Args) -> Self::Output {
                self.call(args)
            }
        }

        #[cfg(feature = "fn_traits")]
        impl std::ops::Fn<$Args> for Function {
            extern "rust-call" fn call(&self, $args: $Args) -> Self::Output {
                let fn_ptr = unsafe { self.$as_fn_ptr() };
                fn_ptr($($args_expanded)*)
            }
        }
    }
}

impl_fn_traits!(
    (),
    as_fn_ptr_0,
    _args,
    ()
);
impl_fn_traits!(
    (c_long, ),
    as_fn_ptr_1,
    args,
    (args.0)
);
impl_fn_traits!(
    (c_long, c_long, ),
    as_fn_ptr_2,
    args,
    (args.0, args.1)
);
impl_fn_traits!(
    (c_long, c_long, c_long, ),
    as_fn_ptr_3,
    args,
    (args.0, args.1, args.2)
);
impl_fn_traits!(
    (c_long, c_long, c_long, c_long, ),
    as_fn_ptr_4,
    args,
    (args.0, args.1, args.2, args.3)
);
impl_fn_traits!(
    (c_long, c_long, c_long, c_long, c_long, ),
    as_fn_ptr_5,
    args,
    (args.0, args.1, args.2, args.3, args.4)
);
impl_fn_traits!(
    (c_long, c_long, c_long, c_long, c_long, c_long, ),
    as_fn_ptr_6,
    args,
    (args.0, args.1, args.2, args.3, args.4, args.5)
);

#[cfg(test)]
mod tests {
    #[test]
    fn add_3() {
        use super::*;
        let f = Function::new(vec![
            PUSH_A.clone(),
            PUSH_B.clone(),
            PUSH_C.clone(),
            ADD.clone(),
            ADD.clone(),
        ]).unwrap();

        let f_ptr = unsafe { f.as_fn_ptr_3() };
        assert_eq!(f_ptr(3, 4, 5), 12);

        #[cfg(feature = "fn_traits")]
        {
            assert_eq!(f(3, 4, 5), 12);
        }

        drop(f);
    }

    #[test]
    fn pow() {
        use super::*;
        let f = Function::new(vec![
            PUSH_VALUE(1),
            PUSH_B.clone(),
            WHILE_LOOP(vec![
                PUSH_A.clone(),
                PUSH_STACK_INDEX(-1),
                MULTIPLY.clone(),
                POP_STACK_INDEX(-1),
                PUSH_VALUE(1),
                SUBTRACT.clone(),
            ]).unwrap(),
            PUSH_STACK_INDEX(-1),
        ]).unwrap();

        let f_ptr = unsafe { f.as_fn_ptr_2() };
        assert_eq!(f_ptr(3, 4), 81);

        #[cfg(feature = "fn_traits")]
        {
            assert_eq!(f(3, 4), 81);
        }

        drop(f);
    }

    #[test]
    fn parse_pow() {
        use super::*;
        let f = Function::parse("1 b { a p-1 * s-1 1 - } p-1").unwrap();

        let f_ptr = unsafe { f.as_fn_ptr_2() };
        assert_eq!(f_ptr(3, 4), 81);
        assert_eq!(f_ptr(3, 5), 243);
        assert_eq!(f_ptr(4, 4), 256);
        assert_eq!(f_ptr(5, 200), -7817535966050405663);

        #[cfg(feature = "fn_traits")]
        {
            assert_eq!(f(3, 4), 81);
            assert_eq!(f(3, 5), 243);
            assert_eq!(f(4, 4), 256);
            assert_eq!(f(5, 200), -7817535966050405663);
        }

        drop(f);
    }
}
