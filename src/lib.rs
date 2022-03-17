#![cfg_attr(feature = "fn_traits", feature(unboxed_closures), feature(fn_traits))]

//#[cfg(not(all(target_arch = "x86_64", target_os = "linux", not(feature = "ignore_target"))))]
//std::compile_error!("This library only works on x86_64 Linux.");

pub(crate) mod raw_code;
pub(crate) mod commands;
pub(crate) mod code;
pub mod function;
pub mod ast;
pub mod compiler;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
//        use crate::*;
        assert_eq!(2 + 2, 4);
    }
}
