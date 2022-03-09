use crate::{raw_code::*, code::{Relocatable, Relocation, RelocationKind, Symbol}, function::FunctionCreateError};
use crate::commands::Command;

make_no_value_static!(PUSH_A, push_a_code, 0, 1, 0);
make_no_value_static!(PUSH_B, push_b_code, 0, 1, 0);
make_no_value_static!(PUSH_C, push_c_code, 0, 1, 0);
make_no_value_static!(PUSH_D, push_d_code, 0, 1, 0);
make_no_value_static!(PUSH_E, push_e_code, 0, 1, 0);
make_no_value_static!(PUSH_F, push_f_code, 0, 1, 0);

make_no_value_static!(ADD, add_code, 2, 1, 2);
make_no_value_static!(SUBTRACT, subtract_code, 2, 1, 2);
make_no_value_static!(MULTIPLY, multiply_code, 2, 1, 2);
// make_no_value_static!(DIVIDE, signed_divide_code, 2, 1, 2);
// make_no_value_static!(MOD, signed_mod_code, 2, 1, 2);
make_no_value_static!(UDIVIDE, unsigned_divide_code, 2, 1, 2);
make_no_value_static!(UMOD, unsigned_mod_code, 2, 1, 2);

lazy_static::lazy_static!{
    pub(crate) static ref DIVIDE: Command = {
        let (code, branch_locations) = signed_divide_code();

        let abort_sym = Symbol::abort();

        let relocations = branch_locations.into_iter().map(|loc| {
            Relocation::new(loc, RelocationKind::Jump24, abort_sym.clone(), -8)
        }).collect();

        Command {
            param_count: 2,
            return_count: 1,
            required_stack_depth: 2,
            code: Relocatable {
                data: code.into(),
                symbols: vec![],
                abs_symbols: vec![],
                relocations,
            },
            data: (&[][..]).into(),
        }
    };
    pub(crate) static ref MOD: Command = {
        let (code, branch_locations) = signed_mod_code();

        let abort_sym = Symbol::abort();

        let relocations = branch_locations.into_iter().map(|loc| {
            Relocation::new(loc, RelocationKind::Jump24, abort_sym.clone(), -8)
        }).collect();

        Command {
            param_count: 2,
            return_count: 1,
            required_stack_depth: 2,
            code: Relocatable {
                data: code.into(),
                symbols: vec![],
                abs_symbols: vec![],
                relocations,
            },
            data: (&[][..]).into(),
        }
    };
}

#[allow(non_snake_case)]
pub(crate) fn PUSH_VALUE(value: isize) -> Command {
    let (code, low_value_loc, high_value_loc) = push_value_code();

    let local_constant_symbol = Symbol::new_local();

    let movw_reloc = Relocation::new(
        low_value_loc,
        RelocationKind::Movw,
        local_constant_symbol.clone(),
        0,
    );
    let movt_reloc = Relocation::new(
        high_value_loc,
        RelocationKind::Movt,
        local_constant_symbol.clone(),
        0,
    );
    Command {
        param_count: 0,
        return_count: 1,
        required_stack_depth: 0,
        code: Relocatable {
            data: code.into(),
            symbols: vec![],
            abs_symbols: vec![(local_constant_symbol, value)],
            relocations: vec![movw_reloc, movt_reloc],
        },
        data: Relocatable::default(),
    }
}


#[allow(non_snake_case)]
pub(crate) fn PUSH_STACK_INDEX(stack_index: i32) -> Command {
    let (code, low_value_loc, high_value_loc) = if stack_index >= 0 {
        push_stack_index_code()
    } else {
        push_negative_stack_index_code()
    };

    let local_constant_symbol = Symbol::new_local();

    let movw_reloc = Relocation::new(
        low_value_loc,
        RelocationKind::Movw,
        local_constant_symbol.clone(),
        0,
    );
    let movt_reloc = Relocation::new(
        high_value_loc,
        RelocationKind::Movt,
        local_constant_symbol.clone(),
        0,
    );
    
    let required_stack_depth: usize = if stack_index >= 0 {
        stack_index as usize + 1
    } else {
        (-stack_index) as usize
    };

    Command {
        param_count: 0,
        return_count: 1,
        required_stack_depth,
        code: Relocatable {
            data: code.into(),
            symbols: vec![],
            abs_symbols: vec![(local_constant_symbol, stack_index as isize)],
            relocations: vec![movw_reloc, movt_reloc],
        },
        data: Relocatable::default(),
    }
}

#[allow(non_snake_case)]
pub(crate) fn POP_STACK_INDEX(stack_index: i32) -> Command {
    let (code, low_value_loc, high_value_loc) = if stack_index >= 0 {
        pop_stack_index_code()
    } else {
        pop_negative_stack_index_code()
    };

    let local_constant_symbol = Symbol::new_local();

    let movw_reloc = Relocation::new(
        low_value_loc,
        RelocationKind::Movw,
        local_constant_symbol.clone(),
        0,
    );
    let movt_reloc = Relocation::new(
        high_value_loc,
        RelocationKind::Movt,
        local_constant_symbol.clone(),
        0,
    );
    
    let required_stack_depth: usize = if stack_index >= 0 {
        stack_index as usize + 2
    } else {
        (-stack_index) as usize + 1
    };

    Command {
        param_count: 1,
        return_count: 0,
        required_stack_depth,
        code: Relocatable {
            data: code.into(),
            symbols: vec![],
            abs_symbols: vec![(local_constant_symbol, stack_index as isize)],
            relocations: vec![movw_reloc, movt_reloc],
        },
        data: Relocatable::default(),
    }
}



fn new_while_loop_header_footer() -> (Relocatable, Relocatable) {
    let header_branch_symbol = Symbol::new_local();
    let footer_branch_symbol = Symbol::new_local();

    let (header_code, header_branch_loc) = while_loop_header_code();

    let header_code = Relocatable {
        data: header_code.into(),
        symbols: vec![(header_branch_symbol.clone(), header_code.len())],
        abs_symbols: vec![],
        relocations: vec![Relocation::new(header_branch_loc, RelocationKind::Jump24, footer_branch_symbol.clone(), -8)],
    };


    let (footer_code, footer_branch_loc) = while_loop_footer_code();

    let footer_code = Relocatable {
        data: footer_code.into(),
        symbols: vec![(footer_branch_symbol.clone(), footer_code.len())],
        abs_symbols: vec![],
        relocations: vec![Relocation::new(footer_branch_loc, RelocationKind::Jump24, header_branch_symbol.clone(), -8)],
    };

    (header_code, footer_code)
}

#[allow(non_snake_case)]
pub(crate) fn WHILE_LOOP(commands: Vec<Command>) -> Result<Command, FunctionCreateError> {
    let (mut code, footer_code) = new_while_loop_header_footer();
    let mut data = Relocatable::default();
    let mut required_stack_depth: usize = 1;
    let mut stack_difference: isize = 0;
    for command in commands {
        dbg!(&command);
        let Command {
            param_count: command_params,
            return_count: command_returns,
            required_stack_depth: command_required_depth,
            code: command_code,
            data: command_data,
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

        code += command_code;
        data += command_data;
    }
    code += footer_code;

    if stack_difference != 0 {
        return Err(FunctionCreateError::LoopChangedStackDepth);
    }
    
    Ok(Command {
        param_count: 0,
        return_count: 0,
        required_stack_depth,
        code,
        data, 
    })
}
