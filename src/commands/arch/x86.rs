use crate::raw_code::*;
use std::borrow::Cow;
use crate::commands::Command;

#[allow(non_snake_case)]
pub(crate) fn PUSH_VALUE(value: isize) -> Command {
    let (code, value_loc) = push_value_code();
    let mut code: Vec<u8> = code.to_owned();
    let value: [u8; 4] = unsafe { std::mem::transmute(value) };
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