use crate::code::*;

#[derive(Debug, Clone)]
pub(crate) struct Command {
    pub(crate) param_count: usize,
    pub(crate) return_count: usize,
    pub(crate) required_stack_depth: usize,
    pub(crate) code: Relocatable,
    pub(crate) data: Relocatable,
}

macro_rules! make_no_value_static {
    ($NAME:ident, $code:ident, $params:expr, $returns:expr, $required_depth:expr) => {
        lazy_static::lazy_static! {
            pub(crate) static ref $NAME: Command = Command {
                param_count: $params,
                return_count: $returns,
                required_stack_depth: $required_depth,
                code: $code().into(),
                data: (&[][..]).into(),
            };
        }
    }
}

mod arch;
pub(crate) use arch::*;

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

        if let Ok(command) = super::WHILE_LOOP(vec![]) {
            let code_and_data = command.code + command.data;
            for byte in code_and_data.assemble().unwrap() {
                print!("{:02x} ", byte);
            }
            println!();
        }

    }
}
