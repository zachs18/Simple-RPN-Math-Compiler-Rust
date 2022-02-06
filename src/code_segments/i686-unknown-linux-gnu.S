#include "function_errors.S"

    .text

    .global function_header_code_start
    .global function_header_code_end
function_header_code_start:
    endbr32
    push %ebp
    mov %esp,%ebp

    # old %ebp is at (%ebp)
    # retaddr is at 4(%ebp)
    # return value pointer is at 8(%ebp)
    # arg 1 is at 12(%ebp)
    # arg 2 is at 16(%ebp)
    # arg 3 is at 20(%ebp)
    # arg 4 is at 24(%ebp)
    # arg 5 is at 28(%ebp)
    # arg 6 is at 32(%ebp)
function_header_code_end:

    .global function_footer_code_start
    .global function_footer_code_end
function_footer_code_start:
    pop %eax
    xor %edx,%edx

    mov %ebp,%esp
    pop %ebp

    mov 4(%esp),%ecx # return value pointer
    mov %eax,(%ecx)
    mov %edx,4(%ecx)

    # "In addition to writing the return value to the proper location, the called function is responsible for
    #  popping the implicit pointer argument off the stack and storing it in %eax prior to returning.
    mov %ecx,%eax

    ret $4
function_footer_code_end:

    .global function_abort_code_start
    .global function_abort_code_end
function_abort_code_start:
    mov %ebp,%esp
    pop %ebp

    mov 4(%esp),%ecx # return value pointer
    mov %eax,(%ecx)
    mov %edx,4(%ecx)

    # "In addition to writing the return value to the proper location, the called function is responsible for
    #  popping the implicit pointer argument off the stack and storing it in %eax prior to returning.
    mov %ecx,%eax

    ret $4
function_abort_code_end:


    .global while_loop_header_code_start
    .global while_loop_header_branch_offset_end
    .global while_loop_header_code_end
while_loop_header_code_start:
    mov (%esp),%eax
    test %eax,%eax
    je .+1024 # force rel32
while_loop_header_branch_offset_end:
while_loop_header_code_end:


    .global while_loop_footer_code_start
    .global while_loop_footer_branch_offset_end
    .global while_loop_footer_code_end
while_loop_footer_code_start:
    mov (%esp),%eax
    test %eax,%eax
    jne .+1024 # force rel32
while_loop_footer_branch_offset_end:
while_loop_footer_code_end:


    .global push_a_code_start
    .global push_a_code_end
push_a_code_start:
    push 12(%ebp)
push_a_code_end:

    .global push_b_code_start
    .global push_b_code_end
push_b_code_start:
    push 16(%ebp)
push_b_code_end:

    .global push_c_code_start
    .global push_c_code_end
push_c_code_start:
    push 20(%ebp)
push_c_code_end:

    .global push_d_code_start
    .global push_d_code_end
push_d_code_start:
    push 24(%ebp)
push_d_code_end:

    .global push_e_code_start
    .global push_e_code_end
push_e_code_start:
    push 28(%ebp)
push_e_code_end:

    .global push_f_code_start
    .global push_f_code_end
push_f_code_start:
    push 32(%ebp)
push_f_code_end:


    .global add_code_start
    .global add_code_end
add_code_start:
    pop %eax
    add %eax,(%esp)
add_code_end:

    .global signed_add_checked_code_start
    .global signed_add_checked_code_end
signed_add_checked_code_start:
    pop %eax
    add %eax,(%esp)
# handle signed overflow
    jno 0f
    mov $SignedAdditionOverflow,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
signed_add_checked_code_end:

    .global unsigned_add_checked_code_start
    .global unsigned_add_checked_code_end
unsigned_add_checked_code_start:
    pop %eax
    add %eax,(%esp)
# handle unsigned overflow
    jnc 0f
    mov $UnsignedAdditionOverflow,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
unsigned_add_checked_code_end:


    .global subtract_code_start
    .global subtract_code_end
subtract_code_start:
    pop %eax
    sub %eax,(%esp)
subtract_code_end:

    .global signed_subtract_checked_code_start
    .global signed_subtract_checked_code_end
signed_subtract_checked_code_start:
    pop %eax
    sub %eax,(%esp)
# handle signed overflow
    jnc 0f
    mov $SignedSubtractionOverflow,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
signed_subtract_checked_code_end:

    .global unsigned_subtract_checked_code_start
    .global unsigned_subtract_checked_code_end
unsigned_subtract_checked_code_start:
    pop %eax
    sub %eax,(%esp)
# handle unsigned overflow
    jnc 0f
    mov $UnsignedSubtractionOverflow,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
unsigned_subtract_checked_code_end:


    .global multiply_code_start
    .global multiply_code_end
multiply_code_start:
    pop %eax
    mull (%esp)
    mov %eax,(%esp)
multiply_code_end:

    .global signed_multiply_checked_code_start
    .global signed_multiply_checked_code_end
signed_multiply_checked_code_start:
    pop %eax
    imull (%esp)
# handle signed overflow
    jno 0f
    mov $SignedMultiplicationOverflow,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
    mov %eax,(%esp)
signed_multiply_checked_code_end:

    .global unsigned_multiply_checked_code_start
    .global unsigned_multiply_checked_code_end
unsigned_multiply_checked_code_start:
    pop %eax
    mull (%esp)
# handle unsigned overflow
    jno 0f
    mov $UnsignedMultiplicationOverflow,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
    mov %eax,(%esp)
unsigned_multiply_checked_code_end:

    .global signed_divide_code_start
    .global signed_divide_code_end
signed_divide_code_start:
    pop %ecx
# handle divide by zero
    test %ecx,%ecx
    jnz 0f
    mov $DivideByZero,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
# handle MIN divide by -1
    cmp $-1,%ecx
    jne 0f
    mov $-0x80000000,%eax
    cmp (%esp),%eax
    jne 0f
    mov $DivideMinByNegativeOne,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
    mov (%esp),%eax
    cdq
    idiv %ecx
    mov %eax,(%esp)
signed_divide_code_end:

    .global signed_mod_code_start
    .global signed_mod_code_end
signed_mod_code_start:
    pop %ecx
# handle mod by zero
    test %ecx,%ecx
    jnz 0f
    mov $DivideByZero,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
# handle MIN mod by -1
    cmp $-1,%ecx
    jne 0f
    mov $-0x80000000,%eax
    cmp (%esp),%eax
    jne 0f
    mov $DivideMinByNegativeOne,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
    mov (%esp),%eax
    cdq
    idiv %ecx
    mov %edx,(%esp)
signed_mod_code_end:

# TODO: divmod?

    .global unsigned_divide_code_start
    .global unsigned_divide_code_end
unsigned_divide_code_start:
    pop %ecx
# handle divide by zero
    test %ecx,%ecx
    jnz 0f
    mov $DivideByZero,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
    mov (%esp),%eax
    cdq
    div %ecx
    mov %eax,(%esp)
unsigned_divide_code_end:

    .global unsigned_mod_code_start
    .global unsigned_mod_code_end
unsigned_mod_code_start:
    pop %ecx
# handle divide by zero
    test %ecx,%ecx
    jnz 0f
    mov $DivideByZero,%edx
    mov $function_abort_code_start,%eax
    jmp *%eax
0:
    mov (%esp),%eax
    cdq
    div %ecx
    mov %edx,(%esp)
unsigned_mod_code_end:

# TODO: udivmod?


    .global push_value_code_start
    .global push_value_value_end
    .global push_value_code_end
push_value_code_start:
    push $256 # force imm32
push_value_value_end:
push_value_code_end:


    .global push_stack_index_code_start
    .global push_stack_index_value_end
    .global push_stack_index_code_end
push_stack_index_code_start:
    mov $0,%eax
push_stack_index_value_end:
    mov (%esp,%eax,4),%ecx
    push %ecx
push_stack_index_code_end:


    .global push_negative_stack_index_code_start
    .global push_negative_stack_index_value_end
    .global push_negative_stack_index_code_end
push_negative_stack_index_code_start:
    mov $0,%eax
push_negative_stack_index_value_end:
    mov (%ebp,%eax,4),%ecx
    push %ecx
push_negative_stack_index_code_end:


    .global pop_stack_index_code_start
    .global pop_stack_index_value_end
    .global pop_stack_index_code_end
pop_stack_index_code_start:
    pop %ecx
    mov $0,%eax
pop_stack_index_value_end:
    mov %ecx,(%esp,%eax,4)
pop_stack_index_code_end:


    .global pop_negative_stack_index_code_start
    .global pop_negative_stack_index_value_end
    .global pop_negative_stack_index_code_end
pop_negative_stack_index_code_start:
    pop %ecx
    mov $0,%eax
pop_negative_stack_index_value_end:
    mov %ecx,(%ebp,%eax,4)
pop_negative_stack_index_code_end:
