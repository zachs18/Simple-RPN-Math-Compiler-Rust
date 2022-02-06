use crate::raw_code::*;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub(crate) struct Command {
    pub(crate) param_count: usize,
    pub(crate) return_count: usize,
    pub(crate) required_stack_depth: usize,
    pub(crate) code: Cow<'static, [u8]>,
}

macro_rules! make_no_value_static {
    ($NAME:ident, $code:ident, $params:expr, $returns:expr, $required_depth:expr) => {
        lazy_static::lazy_static! {
            pub(crate) static ref $NAME: Command = Command {
                param_count: $params,
                return_count: $returns,
                required_stack_depth: $required_depth,
                code: Cow::Borrowed($code()),
            };
        }
    }
}

make_no_value_static!(PUSH_A, push_a_code, 0, 1, 0);
make_no_value_static!(PUSH_B, push_b_code, 0, 1, 0);
make_no_value_static!(PUSH_C, push_c_code, 0, 1, 0);
make_no_value_static!(PUSH_D, push_d_code, 0, 1, 0);
make_no_value_static!(PUSH_E, push_e_code, 0, 1, 0);
make_no_value_static!(PUSH_F, push_f_code, 0, 1, 0);

make_no_value_static!(ADD, add_code, 2, 1, 2);
make_no_value_static!(SUBTRACT, subtract_code, 2, 1, 2);
make_no_value_static!(MULTIPLY, multiply_code, 2, 1, 2);
make_no_value_static!(DIVIDE, divide_code, 2, 1, 2);
make_no_value_static!(MOD, mod_code, 2, 1, 2);
make_no_value_static!(UDIVIDE, udivide_code, 2, 1, 2);
make_no_value_static!(UMOD, umod_code, 2, 1, 2);

#[allow(non_snake_case)]
pub(crate) fn PUSH_VALUE(value: isize) -> Command {
    let (code, value_loc) = push_value_code();
    let mut code: Vec<u8> = code.to_owned();
    let value: [u8; 8] = unsafe { std::mem::transmute(value) };
    code[value_loc].copy_from_slice(&value);
    Command {
        param_count: 0,
        return_count: 1,
        required_stack_depth: 0,
        code: Cow::Owned(code),
    }
}

#[allow(non_snake_case)]
pub(crate) fn PUSH_STACK_INDEX(stack_index: i32) -> Command {
    let (code, value_loc) = if stack_index >= 0 {
        push_stack_index_code()
    } else {
        push_negative_stack_index_code()
    };
    let required_stack_depth: usize = if stack_index >= 0 {
        stack_index as usize + 1
    } else {
        (-stack_index) as usize
    };
    let mut code: Vec<u8> = code.to_owned();
    let value: [u8; 4] = unsafe { std::mem::transmute(stack_index) };
    code[value_loc].copy_from_slice(&value);
    Command {
        param_count: 0,
        return_count: 1,
        required_stack_depth,
        code: Cow::Owned(code),
    }
}

#[allow(non_snake_case)]
pub(crate) fn POP_STACK_INDEX(stack_index: i32) -> Command {
    let (code, value_loc) = if stack_index >= 0 {
        pop_stack_index_code()
    } else {
        pop_negative_stack_index_code()
    };
    let required_stack_depth: usize = if stack_index >= 0 {
        stack_index as usize + 2
    } else {
        (-stack_index) as usize + 1
    };
    let mut code: Vec<u8> = code.to_owned();
    let value: [u8; 4] = unsafe { std::mem::transmute(stack_index) };
    code[value_loc].copy_from_slice(&value);
    Command {
        param_count: 1,
        return_count: 0,
        required_stack_depth,
        code: Cow::Owned(code),
    }
}

#[allow(non_snake_case)]
pub(crate) fn WHILE_LOOP(commands: Vec<Command>) -> Result<Command, &'static str> {
    let mut body_code = Vec::new();
    let mut required_stack_depth: usize = 1;
    let mut stack_difference: isize = 0;
    for command in commands {
        let Command {
            param_count: command_params,
            return_count: command_returns,
            required_stack_depth: command_required_depth,
            code: command_code,
        } = command;
        // TODO: handle overflows
        if ((required_stack_depth as isize + stack_difference) as usize) < command_params {
            required_stack_depth = (command_params as isize - stack_difference) as usize;
        }
        if ((required_stack_depth as isize + stack_difference) as usize) < command_required_depth {
            required_stack_depth = (command_required_depth as isize - stack_difference) as usize;
        }

        stack_difference -= command_params as isize;
        stack_difference += command_returns as isize;

        body_code.extend_from_slice(&*command_code)
    }
    let body_len = body_code.len();

    if stack_difference != 0 {
        return Err("Does not currently support while loops that change stack depth");
    }

    let (header_code, header_offset_loc) = while_loop_header_code();
    let mut header_code = header_code.to_owned();
    let header_len = header_code.len();

    let (footer_code, footer_offset_loc) = while_loop_footer_code();
    let mut footer_code = footer_code.to_owned();
    let footer_len = footer_code.len();

    let header_branch_offset: i32 = ((header_len - header_offset_loc.end) + body_len + footer_len) as i32;
    let footer_branch_offset: i32 = -(((footer_len - header_offset_loc.end) + body_len + header_len) as i32);

    let value: [u8; 4] = unsafe { std::mem::transmute(header_branch_offset) };
    header_code[header_offset_loc].copy_from_slice(&value);

    let value: [u8; 4] = unsafe { std::mem::transmute(footer_branch_offset) };
    footer_code[footer_offset_loc].copy_from_slice(&value);

    header_code.append(&mut body_code);
    header_code.append(&mut footer_code);
    
    Ok(Command {
        param_count: 0,
        return_count: 0,
        required_stack_depth,
        code: Cow::Owned(header_code),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn push_a() {
        println!("{:?}", &*super::PUSH_A);
        println!("{:?}", &*super::PUSH_B);
        println!("{:?}", &*super::PUSH_C);
        println!("{:?}", &*super::PUSH_D);
        println!("{:?}", &*super::PUSH_E);
        println!("{:?}", &*super::PUSH_F);

        println!("{:?}", &*super::ADD);
        println!("{:?}", &*super::SUBTRACT);
        println!("{:?}", &*super::MULTIPLY);
        println!("{:?}", &*super::DIVIDE);
        println!("{:?}", &*super::MOD);
        println!("{:?}", &*super::UDIVIDE);
        println!("{:?}", &*super::UMOD);

        println!("{:?}", super::PUSH_VALUE(-45));
        println!("{:?}", super::PUSH_VALUE(45));

        println!("{:?}", super::PUSH_STACK_INDEX(3));
        println!("{:?}", super::PUSH_STACK_INDEX(-3));
        println!("{:?}", super::POP_STACK_INDEX(3));
        println!("{:?}", super::POP_STACK_INDEX(-3));

        println!("{:?}", super::WHILE_LOOP(vec![]));

        if let Ok(super::Command{ code, .. }) = super::WHILE_LOOP(vec![]) {
            for byte in &*code {
                print!("{:02x} ", byte);
            }
            println!();
        }

    }
}
