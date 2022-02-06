use std::convert::TryInto;

macro_rules! make_no_value_code {
    ($name:ident, $start:ident, $end:ident) => {
        pub(crate) fn $name() -> &'static [u8] {
            extern {
                static $start: [u8; 0];
                static $end: [u8; 0];
            }
            let start: *const u8 = unsafe {&$start[..]}.as_ptr();
            let end:   *const u8 = unsafe {&$end[..]  }.as_ptr();
            assert!(start as usize <= end as usize);
            let length: usize = // TODO: Should this use end as isize - start as isize?
                unsafe{end.offset_from(start)}
                    .try_into()
                    .expect("end should follow start");
            unsafe {std::slice::from_raw_parts(start, length)}
        }
    }
}

make_no_value_code!(function_header_code, function_header_code_start, function_header_code_end);
make_no_value_code!(function_footer_code, function_footer_code_start, function_footer_code_end);

make_no_value_code!(push_a_code, push_a_code_start, push_a_code_end);
make_no_value_code!(push_b_code, push_b_code_start, push_b_code_end);
make_no_value_code!(push_c_code, push_c_code_start, push_c_code_end);
make_no_value_code!(push_d_code, push_d_code_start, push_d_code_end);
make_no_value_code!(push_e_code, push_e_code_start, push_e_code_end);
make_no_value_code!(push_f_code, push_f_code_start, push_f_code_end);

make_no_value_code!(add_code, add_code_start, add_code_end);
make_no_value_code!(subtract_code, subtract_code_start, subtract_code_end);
make_no_value_code!(multiply_code, multiply_code_start, multiply_code_end);
make_no_value_code!(signed_divide_code, signed_divide_code_start, signed_divide_code_end);
make_no_value_code!(signed_mod_code, signed_mod_code_start, signed_mod_code_end);
make_no_value_code!(unsigned_divide_code, unsigned_divide_code_start, unsigned_divide_code_end);
make_no_value_code!(unsigned_mod_code, unsigned_mod_code_start, unsigned_mod_code_end);

mod arch;
pub(crate) use arch::*;
