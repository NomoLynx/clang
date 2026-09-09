# CLang to ASM mapping

this file contains uint32 or unsigned int

| CLang | ASM |
| --- | --- |
| load_id | [load id](#load-id) |
| load_id_to_t1 | [load id to t1](#load-id-to-t1) |
| load_id_from_stack | [load id from stack](#load-id-from-stack) |
| load_value_from_memory_to_register | [load value from memory to register](#load-value-from-memory-to-register) |
| return_int | [return int](#return-int) |
| load_address_to_reg | [load address to register](#load-address-to-register) |
| add_assign | [add assign template](#add-assign-template) |
| mul_assign | [mul assign template](#mul-assign-template) |
| div_assign | [div assign template](#div-assign-template) |
| mod_assign | [mod assign template](#mod-assign-template) |
| sub_assign | [sub assign template](#sub-assign-template) |
| left_assign | [left assign template](#left-assign-template) |
| right_assign | [right assign template](#right-assign-template) |
| and_assign | [and assign template](#and-assign-template) |
| xor_assign | [xor assign template](#xor-assign-template) |
| or_assign | [or assign template](#or-assign-template) |
| dereference | [dereference](#dereference) |

## load id from stack

- output: t0

load an id's address from stack to a register (e.g. t0). Because the load is always inside a function, so use fp instead of sp

```asm
    .text 
    addi t0, fp, 144
```

## load value from memory to register

load value from memory to register using frame pointer (fp) and offset

- output: t0

```asm
    .text
    lwu t0, 144(fp)

```

## load id

- output: t0
- input labels: labelx

load an id's address to a register

```asm
    .text 
    __label_load_id:
    la t0, labelx
```

## load id to t1

load id's value as uint to register t1

- output: t1
- impacted_regs: t0

```asm
    .text
    __label_load_id_to_t1:
    la t0, labelx
    lwu t1, 0(t0)
```

## return int

return int like (return i)

```asm
    .text
    __label__return_int:
    la t0, labelx
    lwu a0, 0(t0)
```

## load address to register

load int from t0's address to t1 register

- input: t0
- output: t1

```asm
    .text 
    lwu t1, 0(t0)
```

## add assign template

handle the add assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__add_assign_template:
    lwu t0, 0(t1)
    add t0, t0, t2
    sw t0, 0(t1)
```

## mul assign template

handle the mul assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__mul_assign_template:
    lwu t0, 0(t1)
    mul t0, t0, t2
    sw t0, 0(t1)
```

## div assign template

handle the div assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__div_assign_template:
    lwu t0, 0(t1)
    div t0, t0, t2
    sw t0, 0(t1)
```

## mod assign template

handle the rem assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__rem_assign_template:
    lwu t0, 0(t1)
    rem t0, t0, t2
    sw t0, 0(t1)
```

## sub assign template

handle the sub assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__sub_assign_template:
    lwu t0, 0(t1)
    sub t0, t0, t2
    sw t0, 0(t1)
```

## left assign template

handle the left assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__left_assign_template:
    lwu t0, 0(t1)
    sll t0, t0, t2
    sw t0, 0(t1)
```

## right assign template

handle the right assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__right_assign_template:
    lwu t0, 0(t1)
    srl t0, t0, t2
    sw t0, 0(t1)
```

## and assign template

handle the and assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__and_assign_template:
    lwu t0, 0(t1)
    and t0, t0, t2
    sw t0, 0(t1)
```

## xor assign template

handle the xor assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__xor_assign_template:
    lwu t0, 0(t1)
    xor t0, t0, t2
    sw t0, 0(t1)
```

## or assign template

handle the or assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to t2

- input: t1, t2
- output: t0

```asm
    .text
    __label__or_assign_template:
    lwu t0, 0(t1)
    or t0, t0, t2
    sw t0, 0(t1)
```

## dereference

deference a value, the input is register holding an address, and the output is the same. The output will hold the value of the address

- input: t0
- output: t0

```asm
    .text 
    lwu t0, 0(t0)
```
