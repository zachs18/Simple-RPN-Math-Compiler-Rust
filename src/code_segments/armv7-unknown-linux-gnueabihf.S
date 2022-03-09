#include "function_errors.S"

    .text

    .balign 4
    .global function_header_code_start
    .global function_header_code_end
function_header_code_start:
// Because FunctionResultRaw is a composite type larger than 4 bytes, it is returned in memory. r0 is the address, we store it in r7
    push {r4-r7,fp,lr}  // sp -= 4*6
    mov fp,sp           // frame pointer
    sub sp,sp,#24       // sp -= 4*6
    add r7,sp,#48       // points to passed-on-stack parameters (arg3,4,5)
    ldm r7,{r4-r6}      // read arg3,4,5
    stm sp,{r1-r6}      // store arg0,1,2,3,4,5
    mov r7,r0           // address of return value
function_header_code_end:

    .balign 4
    .global function_footer_code_start
    .global function_footer_code_end
function_footer_code_start:
    pop {r0}
    mov r1,#0
    stm r7,{r0,r1}
    mov r0,r7           // address of return value
    mov sp,fp           // restore stack from frame pointer
    pop {r4-r7,fp,pc}   // restore saved regs and return
function_footer_code_end:

    .balign 4
    .global function_abort_code_start
    .global function_abort_code_end
function_abort_code_start:
// error code is already in r1
    stm r7,{r0,r1}
    mov r0,r7           // return address of return value
    mov sp,fp
    pop {r4-r7,fp,pc}
function_abort_code_end:


    .balign 4
    .global while_loop_header_code_start
    .global while_loop_header_code_branch
    .global while_loop_header_code_end
while_loop_header_code_start:
    ldr r0,[sp]
    cmp r0,#0
while_loop_header_code_branch:
    beq .
while_loop_header_code_end:


    .balign 4
    .global while_loop_footer_code_start
    .global while_loop_footer_code_branch
    .global while_loop_footer_code_end
while_loop_footer_code_start:
    ldr r0,[sp]
    cmp r0,#0
while_loop_footer_code_branch:
    bne .
while_loop_footer_code_end:


    .balign 4
    .global push_a_code_start
    .global push_a_code_end
push_a_code_start:
    ldr r0,[fp,#-24]
    push {r0}
push_a_code_end:

    .balign 4
    .global push_b_code_start
    .global push_b_code_end
push_b_code_start:
    ldr r0,[fp,#-20]
    push {r0}
push_b_code_end:

    .balign 4
    .global push_c_code_start
    .global push_c_code_end
push_c_code_start:
    ldr r0,[fp,#-16]
    push {r0}
push_c_code_end:

    .balign 4
    .global push_d_code_start
    .global push_d_code_end
push_d_code_start:
    ldr r0,[fp,#-12]
    push {r0}
push_d_code_end:

    .balign 4
    .global push_e_code_start
    .global push_e_code_end
push_e_code_start:
    ldr r0,[fp,#-8]
    push {r0}
push_e_code_end:

    .balign 4
    .global push_f_code_start
    .global push_f_code_end
push_f_code_start:
    ldr r0,[fp,#-4]
    push {r0}
push_f_code_end:


    .balign 4
    .global add_code_start
    .global add_code_end
add_code_start:
    pop {r0,r1}
    add r0,r0,r1
    push {r0}
add_code_end:

    .balign 4
    .global signed_add_checked_code_start
    .global signed_add_checked_code_branch
    .global signed_add_checked_code_end
signed_add_checked_code_start:
    pop {r0,r1}
    adds r0,r0,r1
    push {r0}
// handle signed overflow
    movvs r1,$SignedAdditionOverflow
signed_add_checked_code_branch:
    bvs .  // TODO: add function_abort_code as a code relocatable and set up branching to it
signed_add_checked_code_end:

    .balign 4
    .global unsigned_add_checked_code_start
    .global unsigned_add_checked_code_end
unsigned_add_checked_code_start:
    pop {r0,r1}
    adds r0,r0,r1
    push {r0}
// handle signed overflow
    movcs r1,$UnsignedAdditionOverflow
    bcs .
unsigned_add_checked_branch_offset_end:
unsigned_add_checked_code_end:


    .balign 4
    .global subtract_code_start
    .global subtract_code_end
subtract_code_start:
    pop {r0,r1}
    sub r0,r1,r0
    push {r0}
subtract_code_end:

    .balign 4
    .global signed_subtract_checked_code_start
    .global signed_subtract_checked_code_end
signed_subtract_checked_code_start:
    pop {r0,r1}
    sub r0,r1,r0
    push {r0}
// handle signed overflow
    movvs r1,$SignedSubtractionOverflow
    bvs .  // TODO: add function_abort_code as a code relocatable and set up branching to it
signed_subtract_checked_branch_offset_end:
signed_subtract_checked_code_end:

    .balign 4
    .global unsigned_subtract_checked_code_start
    .global unsigned_subtract_checked_branch
    .global unsigned_subtract_checked_code_end
unsigned_subtract_checked_code_start:
    pop {r0,r1}
    sub r0,r1,r0
    push {r0}
// handle signed overflow
    movcs r1,$UnsignedSubtractionOverflow
unsigned_subtract_checked_branch:
    bcs .
unsigned_subtract_checked_code_end:


    .balign 4
    .global multiply_code_start
    .global multiply_code_end
multiply_code_start:
    pop {r0,r1}
    mul r0,r1,r0
    push {r0}
multiply_code_end:

    .balign 4
    .global signed_multiply_checked_code_start
    .global signed_multiply_checked_code_end
signed_multiply_checked_code_start:
    pop {r2,r3}
    smull r0,r1,r2,r3 // r0 is low, r1 is high
    push {r0}
    mov r2,r0,ASR#31 // sign-extend r0 into r2
    cmp r1,r2 // ensure r1 is sign-extend of r0, i.e. r0 = r1:r0
// handle signed overflow
    movne r1,$SignedMultiplicationOverflow
    bne .
signed_multiply_checked_branch_offset_end:
signed_multiply_checked_code_end:

    .balign 4
    .global unsigned_multiply_checked_code_start
    .global unsigned_multiply_checked_code_end
unsigned_multiply_checked_code_start:
    pop {r2,r3}
    umull r0,r1,r2,r3 // r0 is low, r1 is high
    push {r0}
    cmp r1,#0 // ensure r1 is zero-extend of r0, i.e. r1 = 0
// handle signed overflow
    movne r1,$UnsignedMultiplicationOverflow
    bne .
unsigned_multiply_checked_branch_offset_end:
unsigned_multiply_checked_code_end:

    .balign 4
    .global signed_divide_code_start
    .global signed_divide_branch_1
    .global signed_divide_branch_2
    .global signed_divide_code_end
signed_divide_code_start:
    pop {r1,r2}     // r2 divided by r1
// handle divide by zero
    tst r1,r1
    moveq r1,$DivideByZero
signed_divide_branch_1:
    beq . // branch to abort code
// handle MIN / -1
    mov r3,#0x80000000
    cmp r1,#-1
    cmpeq r2,r3
    moveq r1,$DivideMinByNegativeOne
signed_divide_branch_2:
    beq . // branch to abort code
// division
    @ sdiv r0,r2,r1 // not supported on this processor
// r1,r2 stay
// r4 = remainder = abs(r2)
// r3 = divisor = abs(r1)
// r5 = quotient
    mov r4,r2
    cmp r4,#0
    rsblt r4,r4,#0 // if (r4 < 0) r4 = 0 - r4;
    mov r3,r1
    cmp r3,#0
    rsblt r3,r3,#0 // if (r3 < 0) r3 = 0 - r3;
    mov r5,#0
0:  // while (remainder > divisor)
    cmp r4,r3
    ble 1f
    add r5,r5,#1 // ++quotient;
    sub r4,r4,r3 // remainder -= divisor
    b 0b
1:
// now, make things the right sign
// if dividend < 0 then remainder = -remainder;
    cmp r2,#0
    rsblt r4,r4,#0
// if dividend < 0 XOR divisor < 0 then quotient = -quotient;
// same as if (dividend XOR divisor) < 0
    eor r6,r1,r2
    cmp r6,#0
    rsblt r5,r5,#0
// push quotient
    push {r5}
signed_divide_code_end:

    .balign 4
    .global signed_mod_code_start
    .global signed_mod_branch_1
    .global signed_mod_branch_2
    .global signed_mod_code_end
signed_mod_code_start:
    pop {r1,r2}     // r2 divided by r1
// handle divide by zero
    tst r1,r1
    moveq r1,$DivideByZero
signed_mod_branch_1:
    beq . // branch to abort code
// handle MIN / -1
    mov r3,#0x80000000
    cmp r1,#-1
    cmpeq r2,r3
    moveq r1,$DivideMinByNegativeOne
signed_mod_branch_2:
    beq . // branch to abort code
// division
    @ sdiv r0,r2,r1 // not supported on this processor
// r1,r2 stay
// r4 = remainder = abs(r2)
// r3 = divisor = abs(r1)
// r5 = quotient
    mov r4,r2
    cmp r4,#0
    rsblt r4,r4,#0 // if (r4 < 0) r4 = 0 - r4;
    mov r3,r1
    cmp r3,#0
    rsblt r3,r3,#0 // if (r3 < 0) r3 = 0 - r3;
    mov r5,#0
0:  // while (remainder > divisor)
    cmp r4,r3
    ble 1f
    add r5,r5,#1 // ++quotient;
    sub r4,r4,r3 // remainder -= divisor
    b 0b
1:
// now, make things the right sign
// if dividend < 0 then remainder = -remainder;
    cmp r2,#0
    rsblt r4,r4,#0
// if dividend < 0 XOR divisor < 0 then quotient = -quotient;
// same as if (dividend XOR divisor) < 0
    eor r6,r1,r2
    cmp r6,#0
    rsblt r5,r5,#0
// push remainder
    push {r4}
signed_mod_code_end:

# TODO: divmod?

    .balign 4
    .global unsigned_divide_code_start
    .global unsigned_divide_code_end
unsigned_divide_code_start:
    udf
#    pop %rcx
## handle divide by zero
#    test %rcx,%rcx
#    jnz 0f
#    mov $DivideByZero,%edx
#    movabs $function_abort_code_start,%rax
#    jmp *%rax
#0:
#    mov (%rsp),%rax
#    cqo
#    div %rcx
#    mov %rax,(%rsp)
unsigned_divide_code_end:

    .balign 4
    .global unsigned_mod_code_start
    .global unsigned_mod_code_end
unsigned_mod_code_start:
    udf
#    pop %rcx
## handle divide by zero
#    test %rcx,%rcx
#    jnz 0f
#    mov $DivideByZero,%edx
#    movabs $function_abort_code_start,%rax
#    jmp *%rax
#0:
#    mov (%rsp),%rax
#    cqo
#    div %rcx
#    mov %rdx,(%rsp)
unsigned_mod_code_end:

# TODO: udivmod?


    .balign 4
    .global push_value_code_start
    .global push_value_movw
    .global push_value_movt
    .global push_value_code_end
push_value_code_start:
push_value_movw:
    movw r0,#0
push_value_movt:
    movt r0,#0
    push {r0}
push_value_code_end:


    .balign 4
    .global push_stack_index_code_start
    .global push_stack_index_movw
    .global push_stack_index_movt
    .global push_stack_index_code_end
push_stack_index_code_start:
push_stack_index_movw:
    movw r1,#0
push_stack_index_movt:
    movt r1,#0
    ldr r0,[sp,r1,lsl#2]
    push {r0}
push_stack_index_code_end:


    .balign 4
    .global push_negative_stack_index_code_start
    .global push_negative_stack_index_movw
    .global push_negative_stack_index_movt
    .global push_negative_stack_index_code_end
push_negative_stack_index_code_start:
push_negative_stack_index_movw:
    movw r1,#0
push_negative_stack_index_movt:
    movt r1,#0
    sub r1,r1,#6 // fp is 24 above conceptual stack
    ldr r0,[fp,r1,lsl#2]
    push {r0}
push_negative_stack_index_code_end:


    .balign 4
    .global pop_stack_index_code_start
    .global pop_stack_index_movw
    .global pop_stack_index_movt
    .global pop_stack_index_code_end
pop_stack_index_code_start:
pop_stack_index_movw:
    movw r1,#0
pop_stack_index_movt:
    movt r1,#0
    pop {r0}
    str r0,[sp,r1,lsl#2]
pop_stack_index_code_end:


    .balign 4
    .global pop_negative_stack_index_code_start
    .global pop_negative_stack_index_movw
    .global pop_negative_stack_index_movt
    .global pop_negative_stack_index_code_end
pop_negative_stack_index_code_start:
pop_negative_stack_index_movw:
    movw r1,#0
pop_negative_stack_index_movt:
    movt r1,#0
    sub r1,r1,#6 // fp is 24 above conceptual stack
    pop {r0}
    str r0,[fp,r1,lsl#2]
pop_negative_stack_index_code_end:
