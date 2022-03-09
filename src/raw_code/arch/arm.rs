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
make_no_value_code!(function_abort_code, function_abort_code_start, function_abort_code_end);

make_no_value_code!(push_a_code, push_a_code_start, push_a_code_end);
make_no_value_code!(push_b_code, push_b_code_start, push_b_code_end);
make_no_value_code!(push_c_code, push_c_code_start, push_c_code_end);
make_no_value_code!(push_d_code, push_d_code_start, push_d_code_end);
make_no_value_code!(push_e_code, push_e_code_start, push_e_code_end);
make_no_value_code!(push_f_code, push_f_code_start, push_f_code_end);

make_no_value_code!(add_code, add_code_start, add_code_end);
make_no_value_code!(subtract_code, subtract_code_start, subtract_code_end);
make_no_value_code!(multiply_code, multiply_code_start, multiply_code_end);
// make_no_value_code!(signed_divide_code, signed_divide_code_start, signed_divide_code_end);
// make_no_value_code!(signed_mod_code, signed_mod_code_start, signed_mod_code_end);
make_no_value_code!(unsigned_divide_code, unsigned_divide_code_start, unsigned_divide_code_end);
make_no_value_code!(unsigned_mod_code, unsigned_mod_code_start, unsigned_mod_code_end);

macro_rules! make_value_code {
    ($name:ident, $start:ident, $movw:ident, $movt:ident, $end:ident) => {
        pub(crate) fn $name() -> (&'static [u8], usize, usize) {
            extern {
                static $start: [u8; 0];
                static $movw: [u8; 0];
                static $movt: [u8; 0];
                static $end: [u8; 0];
            }
            let start: *const u8 = unsafe {&$start[..]}.as_ptr();
            let movw:  *const u8 = unsafe {&$movw[..] }.as_ptr();
            let movt:  *const u8 = unsafe {&$movt[..] }.as_ptr();
            let end:   *const u8 = unsafe {&$end[..]  }.as_ptr();
            assert!(start as usize <= movw as usize);
            assert!(movw as usize + 4 <= movt as usize);
            assert!(movt as usize + 4 <= end as usize);
            let movw_index: usize =
                unsafe{movw.offset_from(start)}
                    .try_into()
                    .expect("movw should follow start");
            let movt_index: usize =
                unsafe{movt.offset_from(start)}
                    .try_into()
                    .expect("movt should follow start");
            let length: usize = // TODO: Should this use end as isize - start as isize?
                unsafe{end.offset_from(start)}
                    .try_into()
                    .expect("end should follow start");
            (unsafe {std::slice::from_raw_parts(start, length)}, movw_index, movt_index)
        }
    }
}

make_value_code!(push_value_code, push_value_code_start, push_value_movw, push_value_movt, push_value_code_end);

make_value_code!(push_stack_index_code, push_stack_index_code_start, push_stack_index_movw, push_stack_index_movt, push_stack_index_code_end);
make_value_code!(push_negative_stack_index_code, push_negative_stack_index_code_start, push_negative_stack_index_movw, push_negative_stack_index_movt, push_negative_stack_index_code_end);
make_value_code!(pop_stack_index_code, pop_stack_index_code_start, pop_stack_index_movw, pop_stack_index_movt, pop_stack_index_code_end);
make_value_code!(pop_negative_stack_index_code, pop_negative_stack_index_code_start, pop_negative_stack_index_movw, pop_negative_stack_index_movt, pop_negative_stack_index_code_end);

macro_rules! make_branch_code {
    ($name:ident, $start:ident, $branch:ident, $end:ident) => {
        pub(crate) fn $name() -> (&'static [u8], usize) {
            extern {
                static $start: [u8; 0];
                static $branch: [u8; 0];
                static $end: [u8; 0];
            }
            let start: *const u8 = unsafe {&$start[..]}.as_ptr();
            let branch:  *const u8 = unsafe {&$branch[..] }.as_ptr();
            let end:   *const u8 = unsafe {&$end[..]  }.as_ptr();
            assert!(start as usize <= branch as usize);
            assert!(branch as usize + 4 <= end as usize);
            let branch_index: usize =
                unsafe{branch.offset_from(start)}
                    .try_into()
                    .expect("branch should follow start");
            let length: usize = // TODO: Should this use end as isize - start as isize?
                unsafe{end.offset_from(start)}
                    .try_into()
                    .expect("end should follow start");
            (unsafe {std::slice::from_raw_parts(start, length)}, branch_index)
        }
    };
    ($name:ident, $start:ident, ( $($branches:ident),*), $end:ident) => {
        pub(crate) fn $name() -> (&'static [u8], Vec<usize>) {
            extern {
                static $start: [u8; 0];
                $( static $branches: [u8; 0]; )*
                static $end: [u8; 0];
            }
            let start: *const u8 = unsafe {&$start[..]}.as_ptr();
            let branches = [ $( unsafe {&$branches[..] }.as_ptr() ),* ];
            let end:   *const u8 = unsafe {&$end[..]  }.as_ptr();
            assert!(start as usize <= branches[0] as usize);
            for (b1, b2) in branches.iter().take(branches.len()-1).zip(branches.iter().skip(1)) {
                assert!(*b1 as usize + 4 <= *b2 as usize);
            }
            assert!(branches[branches.len()-1] as usize + 4 <= end as usize);
            let branch_indices = branches.map(|branch|
                unsafe{branch.offset_from(start)}
                    .try_into()
                    .expect("branch should follow start")
            );
            let length: usize = // TODO: Should this use end as isize - start as isize?
                unsafe{end.offset_from(start)}
                    .try_into()
                    .expect("end should follow start");
            (unsafe {std::slice::from_raw_parts(start, length)}, branch_indices.into())
        }
    };
}

make_branch_code!(while_loop_header_code, while_loop_header_code_start, while_loop_header_code_branch, while_loop_header_code_end);
make_branch_code!(while_loop_footer_code, while_loop_footer_code_start, while_loop_footer_code_branch, while_loop_footer_code_end);

make_branch_code!(signed_divide_code, signed_divide_code_start, (signed_divide_branch_1, signed_divide_branch_2), signed_divide_code_end);
make_branch_code!(signed_mod_code, signed_mod_code_start, (signed_mod_branch_1, signed_mod_branch_2), signed_mod_code_end);
