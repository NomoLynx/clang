# CLang to ASM mapping

labelx and 144 is the value to be replaced.

| CLang | ASM |
| --- | --- |
| exit | [main_asm](#main_asm) |
| main_asm_return0 | [main_asm_return0](#main_asm_return0) |
| function_template | [function template](#function-template) |
| global_function_template | [global function template](#global-function-template) |
| compound_template | [compound template](#compound-template) |
| function_return | [function return](#function-return) |
| function_call | [function call](#function-call) |
| indirect_function_call | [indirect function call](#indirect-function-call) |
| load_int | [load int](#load-int) |
| save_int | [save int](#save-int) |
| save_value_to_address | [save value to address](#save-value-to-address) |
| load_const_to_a0 | [load const to a0](#load-const-to-a0) |
| load_zero_x0 | [load zero x0](#load-zero-x0) |
| load_id_to_t1 | [load id to t1](#load-id-to-t1) |
| load_composite_id_to_t1 | [load composite id to t1](#load-composite-id-to-t1) |
| load_value_from_memory_to_register | [load value from memory to register](#load-value-from-memory-to-register) |
| return_int | [return int](#return-int) |
| add | [add operation](#add-operation) |
| sub | [sub operation](#sub-operation) |
| mul | [mul operation](#mul-operation) |
| div | [div operation](#div-operation) |
| rem | [rem operation](#rem-operation) |
| xor | [xor operation](#xor-operation) |
| and | [and operation](#and-operation) |
| or | [or operation](#or-operation) |
| sll | [sll operation](#sll-operation) |
| srl | [srl operation](#srl-operation) |
| boolean_operation | [boolean operation](#boolean-operation) |
| not_equal_branch | [not equal branch](#not-equal-branch) |
| equal_branch | [equal_branch](#equal_branch) |
| greater_branch | [greater_branch](#greater_branch) |
| less_branch | [less_branch](#less_branch) |
| greater_equal_branch | [greater_equal_branch](#greater_equal_branch) |
| less_equal_branch | [less_equal_branch](#less_equal_branch) |
| load_id | [load id](#load-id) |
| load_id_from_stack | [load id from stack](#load-id-from-stack) |
| load_address_to_reg | [load address to register](#load-address-to-register) |
| neg | [neg](#neg) |
| neg_no_save | [neg_no_save](#neg_no_save) |
| inc | [inc](#inc) |
| inc_no_save | [inc_no_save](#inc_no_save) |
| desc | [desc](#desc) |
| desc_no_save | [desc_no_save](#desc_no_save) |
| bitwise_not | [bitwise_not](#bitwise_not) |
| logic_not | [logic_not](#logic_not) |
| if_only | [if only template](#if-only-template) |
| if_else | [if else template](#if-else-template) |
| for_statement_template | [for_statement_template](#for_statement_template) |
| do_while_template | [do_while_template](#do_while_template) |
| while_template | [while_template](#while_template) |
| add_assign | [add assign template](#add-assign-template) |
| mul_assign | [mul assign template](#mul-assign-template) |
| mul_const | [mul const operation](#mul-const-operation) |
| div_assign | [div assign template](#div-assign-template) |
| mod_assign | [mod assign template](#mod-assign-template) |
| sub_assign | [sub assign template](#sub-assign-template) |
| left_assign | [left assign template](#left-assign-template) |
| right_assign | [right assign template](#right-assign-template) |
| and_assign | [and assign template](#and-assign-template) |
| xor_assign | [xor assign template](#xor-assign-template) |
| or_assign | [or assign template](#or-assign-template) |
| add_assign_no_reg | [add assign no reg template](#add-assign-no-reg-template) |
| mul_assign_no_reg | [mul assign no reg template](#mul-assign-no-reg-template) |
| div_assign_no_reg | [div assign no reg template](#div-assign-no-reg-template) |
| mod_assign_no_reg | [mod assign no reg template](#mod-assign-no-reg-template) |
| sub_assign_no_reg | [sub assign no reg template](#sub-assign-no-reg-template) |
| left_assign_no_reg | [left assign no reg template](#left-assign-no-reg-template) |
| right_assign_no_reg | [right assign no reg template](#right-assign-no-reg-template) |
| and_assign_no_reg | [and assign no reg template](#and-assign-no-reg-template) |
| xor_assign_no_reg | [xor assign no reg template](#xor-assign-no-reg-template) |
| or_assign_no_reg | [or assign no reg template](#or-assign-no-reg-template) |
| case_template | [case template](#case-template) |
| default_template | [default template](#default-template) |
| switch_template | [switch template](#switch-template) |
| break_template | [break template](#break-template) |
| continue_template | [continue template](#continue-template) |
| goto_template | [goto template](#goto-template) |
| save_stack_pointer | [save stack pointer](#save-stack-pointer) |
| restore_stack_pointer | [restore stack pointer](#restore-stack-pointer) |
| set_local_variable_to_stack | [set local variable to stack](#set-local-variable-to-stack) |
| push_local_varialbe_with_value_to_stack | [push local varialbe with value to stack](#push-local-varialbe-with-value-to-stack) |
| save_local_variable_value | [save local variable value](#save-local-variable-value) |
| dereference | [dereference](#dereference) |
| move_register | [move register](#move-register) |
| extern | [extern](#extern) |
| array_definition | [array definition](#array-definition) |
| zero_register | [zero a register](#zero-a-register) |
| offset_from_array_index | [offset from array index](#offset-from-array-index) |
| add_stack_fp_and_offset | [add stack fp and offset](#add-stack-fp-and-offset) |
| allocate_stack_space | [allocate stack space](#allocate-stack-space) |
| stack_definition | [stack definition](#stack-definition) |

please notes that: labelx and 144 will be replaced.

## main_asm

asm code for return which return a0 as return code

- input : a0

```asm
    .text
    __label__exit:
    addi a7, x0, 93
    ecall

```

## main_asm_return0

asm code for return which return 0 as return code

```asm
    .text
    __label__exit:
    addi a0, x0, 0
    addi a7, x0, 93
    ecall

```

## function template

asm code for a function (not main)

```asm
    .text
    labelx:    # this will be the function name

    __parameters__to__stack:

    __function__body:
```

## global function template

asm code for a global function

```asm
    .text
    .globl  labelx
    .type   labelx, @function
    labelx:    # this will be the function name

    __parameters__to__stack:

    __function__body:

    __function_body_end:
    .size   labelx, .-labelx
```

## compound template

asm code for compound statement, it can be a function body, for body, etc. it can define local variable

```asm
    .text

```

## function return

return from a function, this statement used for the RETURN statement in a function

```asm
    .text
    __label__exit:
    ret
```

## function call

call a function, the function name will be at labelx's location. fp is the register points to a fixed location during the function call. ra is the return address register.

- fixed_regs: sp
- impact_regs: t0
- output: a0

```asm
    .text
    push64 ra
    push64 fp
    __label__parameters__onto__stack:

    # 144 will be replaced by the #_of_parameter * 8, cannot do fp=sp after push64 sp, 
    # because parameter loading will need use old/current fp fp=sp, will contaminate the fp value and cannot load parameters
    # sets fp to the stack position before argument pushes (but after saved ra/fp), so after the call the caller can restore stack in one step via mv sp, fp, then pop64 fp, pop64 ra
    addi fp, sp, 144 
    call labelx
    #no need to pop parameters, parameters will be pop out anyway and no need to worry
    mv sp, fp  #restore sp location from fp
    __label__restore__after__call:
    pop64 fp
    pop64 ra
```

## indirect function call

indirect call a function, fp is the register points to a fixed location during the function call. ra is the return address register.

- input : t6
- fixed_regs: sp
- impact_regs: t0
- output: a0

```asm
    .text
    push64 ra
    push64 fp
    __label__parameters__onto__stack:

    # 144 will be replaced by the #_of_parameter * 8, cannot do fp=sp after push64 sp, 
    # because parameter loading will need use old/current fp fp=sp, will contaminate the fp value and cannot load parameters
    # sets fp to the stack position before argument pushes (but after saved ra/fp), so after the call the caller can restore stack in one step via mv sp, fp, then pop64 fp, pop64 ra
    addi fp, sp, 144 
    jalr ra, t6, 0
    #no need to pop parameters, parameters will be pop out anyway and no need to worry
    mv sp, fp  #restore sp location from fp
    __label__restore__after__call:
    pop64 fp
    pop64 ra
```

## load int

load int to temp register t0 and then save to labelX

```asm
    .text
    __label__load_int_to_t0:
    la t0, labelx
    li t1, 144
    sw t1, 0(t0)
```

## save int

save int to labelx memroy location

```asm
    .text
    __label__save_int_to_t0:
    la t0, labelx
    sw t1, 0(t0)
```

## save value to address

- input : t1, t0
- output: t0

save int to a address which is stored in a register already

```asm
    .text 
    sw t0, 0(t1)
```

## load const to a0

load const to regsiter t3

- output: t3

```asm
    .text
    li t3, 144
```

## load zero x0

load zero const and it is x0

- output: x0

```asm

    .text 

```

## load id to t1

load id's value as int to register t1

- output: t1
- impacted_regs: t0

```asm
    .text
    la t0, labelx
    lw t1, 0(t0)
```

## load composite id to t1

load composite id as int to register t1

- output: t1
- impact_regs: t0

```asm
    .text
    __label_load_composte_id_to_t1:
    la t0, labelx
    addi t0, t0, 144
    lw t1, 0(t0)

```

## load value from memory to register

load value from memory to register using frame pointer (fp) and offset

- output: t0

```asm
    .text
    lw t0, 144(fp)

```

## return int

return int like (return i)

```asm
    .text
    __label__return_int:
    la t0, labelx
    lw a0, 0(t0)
```

## not equal branch

- input: t0, t1
- output: t2

```asm
    .text
    mv t2, x0
    beq t0, t1, __end__notequal
    addi t2, t2, 1
    __end__notequal:
```

## equal_branch

- input: t0, t1
- output: t2

```asm
    .text
    mv t2, x0
    bne t0, t1, __end__equal
    addi t2, t2, 1
    __end__equal:
```

## greater_branch

- input: t0, t1
- output: t2

```asm
    .text
    mv t2, x0
    ble t0, t1, __end__greater
    addi t2, t2, 1
    __end__greater:
```

## less_branch

- input: t0, t1
- output: t2

```asm
    .text
    mv t2, x0
    bge t0, t1, __end__less
    addi t2, t2, 1
    __end__less:
```

## greater_equal_branch

- input: t0, t1
- output: t2

```asm
    .text
    mv t2, x0
    blt t0, t1, __end__greater_equal
    addi t2, t2, 1
    __end__greater_equal:
```

## less_equal_branch

- input: t0, t1
- output: t2

```asm
    .text
    mv t2, x0
    bgt t0, t1, __end__less_equal
    addi t2, t2, 1
    __end__less_equal:
```

## add operation

this template also used add.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_add_operation:
    add t2, t0, t1
```

## sub operation

this template also used sub.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_sub_operation:
    sub t2, t0, t1
```

## mul operation

this template also used mul.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_mul_operation:
    mul t2, t0, t1
```

## mul const operation

this template also used mul a const. t2 set to t2 x 144 using t1 as bridge register.

- input: t2
- output: t2
- temp : t1
- impact_regs: t1

```asm
    .text 
    ___label_muli_operation:
    muli t2, t1, 144
```

## div operation

this template also used div.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_div_operation:
    div t2, t0, t1
```

## rem operation

this template also used rem.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_rem_operation:
    rem t2, t0, t1
```

## xor operation

this template also used xor.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_xor_operation:
    xor t2, t0, t1
```

## and operation

this template also used and.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_and_operation:
    and t2, t0, t1
```

## or operation

this template also used or.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_rem_operation:
    or t2, t0, t1
```

## sll operation

this template also used sll.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_sll_operation:
    sll t2, t0, t1
```

## srl operation

this template also used srl.

- input: t0, t1
- output: t2

```asm
    .text 
    ___label_srl_operation:
    srl t2, t0, t1
```

## boolean operation

this template is for operaiton generate Boolean values, like beq, eq, etc.

- input: t0, t1
- output: t2

```asm
    .text
    ___boolean_operation:
    add t2, t0, t1
```

## neg

template for -var0

- input: t1, t0
- output: t1

```asm
    .text
    sub t1, x0, t1
    sw t1, 0(t0)
```

## neg_no_save

template for -var0

- input: t1
- output: t1

```asm
    .text
    sub t1, x0, t1
```

## inc

template for var0++

- input : t1, t0
- output: t1

```asm
    .text 
    __label_inc_template:
    addi t1, t1, 144
    sw t1, 0(t0)
```

## inc_no_save

template for var0++

- input : t1
- output: t1

```asm
    .text 
    __label_inc_no_save_template:
    addi t1, t1, 144
```

## desc

template for var0--

- input : t1, t0
- output: t1

```asm
    .text 
    __label_inc_template:
    addi t1, t1, 144
    sw t1, 0(t0)
```

## desc_no_save

template for var0-- and the value is stored in register

- input : t1
- output: t1

```asm
    .text 
    __label_inc_no_save_template:
    addi t1, t1, 144
```

## bitwise_not

template for bitwise not operation

- input : t0
- output: t0

```asm
    .text
    __label_bitwise_not_template:
    xori t0, t0, -1
```

## logic_not

template for logic not operation

- input: t0
- output: t0

```asm
    .text
    xori t0, t0, -1
    beq t0, x0, __logic_not_end
    li t0, 1
__logic_not_end:

```

## load id

- output: t0
- input labels: labelx

load an id's address to a register

```asm
    .text 
    la t0, labelx
```

## load id from stack

- output: t0

load an id's address from stack to a register (e.g. t0). Because the load is always inside a function, so use fp instead of sp

```asm
    .text 
    addi t0, fp, 144
```

## load address to register

load int value from t0's address to t1 register

- input: t0
- output: t1

```asm
    .text
    lw t1, 0(t0)
```

## if only template

if only template for something like IF bool THEN AAA END. It has not ELSE statement.

- input: t0

```asm
    .text 
    if_only:
        beq t0, x0, end_if
    if_branch:
        # AAA section
    end_if:
```

## if else template

if else template is for someting like IF bool THEN AAA ELSE BBB END.

- input: t0

```asm
    .text 
    if_only:
        beq t0, x0, end_if
    if_branch:
        # AAA section
        jal x0, x0, end_else
    end_if:
        # BBB section
    end_else:
```

## for_statement_template

- input: t0

```asm
    .text
    for_statement_start:
        # init status set
    for_evaluation:
        # conditional to end_if
        beq t0, x0, end_if
    statement:
        # and statements
    status_change:
        # status change

        jal x0, x0, for_evaluation
    end_if:
```

## do_while_template

- input: t0

```asm
    .text 
    do_while_start:
    end_if:
        # statements
        do_condition:
        # conditional to end_if
        beq t0, x0, end_if
    do_while_end:
```

## while_template

- input: t0

```asm
    .text
    while_start:
        # conditional to end_if
        beq t0, x0, end_if
        while_statements:
        # statements
        jal x0, x0, while_start
    end_if:
    while_end:
```

## add assign template

handle the add assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__add_assign_template:
    lw t0, 0(t1)
    add t0, t0, t2
    sw t0, 0(t1)
```

## add assign no reg template

handle the add assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__add_assign_no_reg_template:
    add t0, t0, t2

```

## mul assign template

handle the mul assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__mul_assign_template:
    lw t0, 0(t1)
    mul t0, t0, t2
    sw t0, 0(t1)
```

## mul assign no reg template

handle the mul assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__mul_no_reg_assign_template:
    mul t0, t0, t2

```

## div assign template

handle the div assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__div_assign_template:
    lw t0, 0(t1)
    div t0, t0, t2
    sw t0, 0(t1)
```

## div assign no reg template

handle the div assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__div_no_reg_assign_template:
    div t0, t0, t2

```

## mod assign template

handle the rem assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__rem_assign_template:
    lw t0, 0(t1)
    rem t0, t0, t2
    sw t0, 0(t1)
```

## mod assign no reg template

handle the rem assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__rem_no_reg_assign_template:
    rem t0, t0, t2

```

## sub assign template

handle the sub assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__sub_assign_template:
    lw t0, 0(t1)
    sub t0, t0, t2
    sw t0, 0(t1)
```

## sub assign no reg template

handle the sub assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__sub_no_reg_assign_template:
    sub t0, t0, t2

```

## left assign template

handle the left assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__left_assign_template:
    lw t0, 0(t1)
    sll t0, t0, t2
    sw t0, 0(t1)
```

## left assign no reg template

handle the left assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__left_no_reg_assign_template:
    sll t0, t0, t2

```

## right assign template

handle the right assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__right_assign_template:
    lw t0, 0(t1)
    srl t0, t0, t2
    sw t0, 0(t1)
```

## right assign no reg template

handle the right assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__right_no_reg_assign_template:
    srl t0, t0, t2

```

## and assign template

handle the and assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__and_assign_template:
    lw t0, 0(t1)
    and t0, t0, t2
    sw t0, 0(t1)
```

## and assign no reg template

handle the and assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__and_no_reg_assign_template:
    and t0, t0, t2

```

## xor assign template

handle the xor assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__xor_assign_template:
    lw t0, 0(t1)
    xor t0, t0, t2
    sw t0, 0(t1)
```

## xor assign no reg template

handle the xor assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__xor_no_reg_assign_template:
    xor t0, t0, t2

```

## or assign template

handle the or assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__or_assign_template:
    lw t0, 0(t1)
    or t0, t0, t2
    sw t0, 0(t1)
```

## or assign no reg template

handle the or assgin operator, and the value of i (or constant 2) is loaded to t2

- input: t0, t2
- output: t0

```asm
    .text
    __label__or_no_reg_assign_template:
    or t0, t0, t2

```

## case template

handle the case in a switch statement, The t6 reigister here is to take the load_case section's output. The "load_case_const" and beq instruction will be taken out to form a jump table.

- input: t6, t1
- replace label: false

```asm
    .text
    __lable_case_template:
    load_case_const:
    beq t6, t1, load_case_const
    case_statements:
    end_case:
```

## default template

the default template for switch statement. This template designed similiar like the case template for easy process.

- replace label: false

```asm
    .text
    __label_default_template:
    load_case_const:
    jal x0, x0, load_case_const
    case_statements:
    end_case:
```

## switch template

switch template

- replace lable: false

```asm
    .text 
    switch_start:
    case_section:
    case_statements:
    BREAK_TARGET:
```

## break template

break template

- tags: break

```asm
    .text
    jal x0, x0, BREAK_TARGET
```

## continue template

continue template

- replace label: false
- tags: continue

```asm
    .text 
    jal x0, x0, CONTINUE
```

## goto template

- replace label: false
- tags: goto

```asm
    .text
    jal x0, x0, GOTO

```

## save stack pointer

this code snippet is to save stack pointer

```asm
    .text 
    save sp, t6

```

## restore stack pointer

this code snippet is to restore stack pointer

```asm
    .text 
    restore sp, t6

```

## set local variable to stack

this code snippet is to set local variable onto stack

```asm
    .text
    sw x0, fp, 144

```

## push local varialbe with value to stack

this code snippet is to push local variable in register with initial value onto stack. t0 is the input value. t1 holds fp's value

- input: t0, t1

```asm
    .text
    pushby t1, t0
```

## save local variable value

this code snippet is to save local variable value to stack. fp is the stack pointer and 144 will replaced with actual stack offset

- input: t0

```asm
    .text
    sw t0, 144(fp)
```

## dereference

deference a value, the input is register holding an address, and the output is the same. The output will hold the value of the address

- input: t0
- output: t1

```asm
    .text 
    lw t1, 0(t0)
```

## move register

move value in one register into another register

- input: t0
- output: t1

```asm

    .text
    mv t1, t0
```

## extern

declare extern symbol

```asm
.data
    .extern labelx

```

## array definition

define array space

```asm
    .data 
    labelx: .space 144

```

## zero a register

set a register's value to zero / x0.

- output: t0

```asm
    .text 
    addi t0, x0, x0
```

## offset from array index

this is one step to compute the offset based on index and array demision. 144 is the dimension which will be set by compiler. t1 is the index from an expression.

- input: t0, t1
- output: t0
- impacted_regs: t2

```asm
    .text 
    li t2, 144
    mul t0, t0, t2
    add t0, t0, t1
```

## add stack fp and offset

add stack fp and offset value

- input: t0
- output: t0

```asm
    .text
    addi t0, t0, 144
    add t0, t0, fp
```

## allocate stack space

allocate stack space.

```asm
    .text
    inc sp, 144
```

## stack definition

declare the stack space in data section.

```asm
    .text
        la sp, tt_stack_end
    .data 
        tt_stack: .space 256
        tt_stack_end:
```
