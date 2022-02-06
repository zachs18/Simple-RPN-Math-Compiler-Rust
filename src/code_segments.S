    .text

    .global function_header_code_start
    .global function_header_code_end
function_header_code_start:
    endbr64
    push %rbp
    mov %rsp,%rbp

    push %rdi # at -8(%rbp)
    push %rsi # at -16(%rbp)
    push %rdx # at -24(%rbp)
    push %rcx # at -32(%rbp)
    push %r8 # at -40(%rbp)
    push %r9 # at -48(%rbp)
function_header_code_end:

    .global function_footer_code_start
    .global function_footer_code_end
function_footer_code_start:
    pop %rax

    mov %rbp,%rsp
    pop %rbp
    ret
function_footer_code_end:


    .global while_loop_header_code_start
    .global while_loop_header_branch_offset_end
    .global while_loop_header_code_end
while_loop_header_code_start:
    mov (%rsp),%rax
    test %rax,%rax
    je .+1024 # force rel32
while_loop_header_branch_offset_end:
while_loop_header_code_end:


    .global while_loop_footer_code_start
    .global while_loop_footer_branch_offset_end
    .global while_loop_footer_code_end
while_loop_footer_code_start:
    mov (%rsp),%rax
    test %rax,%rax
    jne .+1024 # force rel32
while_loop_footer_branch_offset_end:
while_loop_footer_code_end:


    .global push_a_code_start
    .global push_a_code_end
push_a_code_start:
    push -8(%rbp)
push_a_code_end:

    .global push_b_code_start
    .global push_b_code_end
push_b_code_start:
    push -16(%rbp)
push_b_code_end:

    .global push_c_code_start
    .global push_c_code_end
push_c_code_start:
    push -24(%rbp)
push_c_code_end:

    .global push_d_code_start
    .global push_d_code_end
push_d_code_start:
    push -32(%rbp)
push_d_code_end:

    .global push_e_code_start
    .global push_e_code_end
push_e_code_start:
    push -40(%rbp)
push_e_code_end:

    .global push_f_code_start
    .global push_f_code_end
push_f_code_start:
    push -48(%rbp)
push_f_code_end:


    .global add_code_start
    .global add_code_end
add_code_start:
    pop %rax
    add %rax,(%rsp)
add_code_end:

    .global subtract_code_start
    .global subtract_code_end
subtract_code_start:
    pop %rax
    sub %rax,(%rsp)
subtract_code_end:

    .global multiply_code_start
    .global multiply_code_end
multiply_code_start:
    pop %rax
    mulq (%rsp)
    mov %rax,(%rsp)
multiply_code_end:

    .global divide_code_start
    .global divide_code_end
divide_code_start:
# TODO: handle divide by 0, possibly by returning early with an indicative value in rdx
    pop %rcx
    mov (%rsp),%rax
    cqo
    idiv %rcx
    mov %rax,(%rsp)
divide_code_end:

    .global mod_code_start
    .global mod_code_end
mod_code_start:
# TODO: handle mod by 0, possibly by returning early with an indicative value in rdx
    pop %rcx
    mov (%rsp),%rax
    cqo
    idiv %rcx
    mov %rdx,(%rsp)
mod_code_end:

# TODO: divmod?

    .global udivide_code_start
    .global udivide_code_end
udivide_code_start:
# TODO: handle udivide by 0, possibly by returning early with an indicative value in rdx
    pop %rcx
    mov (%rsp),%rax
    cqo
    div %rcx
    mov %rax,(%rsp)
udivide_code_end:

    .global umod_code_start
    .global umod_code_end
umod_code_start:
# TODO: handle umod by 0, possibly by returning early with an indicative value in rdx
    pop %rcx
    mov (%rsp),%rax
    cqo
    div %rcx
    mov %rdx,(%rsp)
umod_code_end:

# TODO: udivmod?


    .global push_value_code_start
    .global push_value_value_end
    .global push_value_code_end
push_value_code_start:
    movabs $0,%rax
push_value_value_end:
    push %rax
push_value_code_end:


    .global push_stack_index_code_start
    .global push_stack_index_value_end
    .global push_stack_index_code_end
push_stack_index_code_start:
    mov $0,%rax
push_stack_index_value_end:
    mov (%rsp,%rax,8),%rcx
    push %rcx
push_stack_index_code_end:


    .global push_negative_stack_index_code_start
    .global push_negative_stack_index_value_end
    .global push_negative_stack_index_code_end
push_negative_stack_index_code_start:
    mov $0,%rax
push_negative_stack_index_value_end:
    mov -48(%rbp,%rax,8),%rcx
    push %rcx
push_negative_stack_index_code_end:


    .global pop_stack_index_code_start
    .global pop_stack_index_value_end
    .global pop_stack_index_code_end
pop_stack_index_code_start:
    pop %rcx
    mov $0,%rax
pop_stack_index_value_end:
    mov %rcx,(%rsp,%rax,8)
pop_stack_index_code_end:


    .global pop_negative_stack_index_code_start
    .global pop_negative_stack_index_value_end
    .global pop_negative_stack_index_code_end
pop_negative_stack_index_code_start:
    pop %rcx
    mov $0,%rax
pop_negative_stack_index_value_end:
    mov %rcx,-48(%rbp,%rax,8)
pop_negative_stack_index_code_end:
