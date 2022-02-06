use std::convert::TryInto;

macro_rules! make_value_code {
    ($name:ident, $start:ident, $value_end:ident, $end:ident, $value_size:expr) => {
        pub(crate) fn $name() -> (&'static [u8], std::ops::Range<usize>) {
            extern {
                static $start: [u8; 0];
                static $value_end: [u8; 0];
                static $end: [u8; 0];
            }
            let start:     *const u8 = unsafe {&$start[..]}.as_ptr();
            let value_end: *const u8 = unsafe {&$value_end[..]}.as_ptr();
            let end:       *const u8 = unsafe {&$end[..]  }.as_ptr();
            assert!(start as usize + $value_size <= value_end as usize && value_end as usize <= end as usize);
            let value_end_idx: usize = // TODO: Should this use end as isize - start as isize?
                unsafe{value_end.offset_from(start)}
                    .try_into()
                    .expect("value_end should follow start");
            let value_idx: usize =
                value_end_idx
                    .checked_sub($value_size)
                    .expect("value_end should be at least value_size bytes after start");
            let length: usize = // TODO: Should this use end as isize - start as isize?
                unsafe{end.offset_from(start)}
                    .try_into()
                    .expect("end should follow start");
            (unsafe {std::slice::from_raw_parts(start, length)}, value_idx .. value_idx+$value_size)
        }
    }
}

make_value_code!(push_value_code, push_value_code_start, push_value_value_end, push_value_code_end, 4);

make_value_code!(push_stack_index_code, push_stack_index_code_start, push_stack_index_value_end, push_stack_index_code_end, 4);
make_value_code!(push_negative_stack_index_code, push_negative_stack_index_code_start, push_negative_stack_index_value_end, push_negative_stack_index_code_end, 4);
make_value_code!(pop_stack_index_code, pop_stack_index_code_start, pop_stack_index_value_end, pop_stack_index_code_end, 4);
make_value_code!(pop_negative_stack_index_code, pop_negative_stack_index_code_start, pop_negative_stack_index_value_end, pop_negative_stack_index_code_end, 4);

make_value_code!(while_loop_header_code, while_loop_header_code_start, while_loop_header_branch_offset_end, while_loop_header_code_end, 4);
make_value_code!(while_loop_footer_code, while_loop_footer_code_start, while_loop_footer_branch_offset_end, while_loop_footer_code_end, 4);
