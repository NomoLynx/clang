# CLang to ASM mapping

this file contains float snippet

| CLang | ASM |
| --- | --- |
| load_const_to_a0 | [load const to a0](#load-const-to-a0) |
| load_id_to_t1 | [load id to t1](#load-id-to-t1) |
| load_value_from_memory_to_register | [load value from memory to register](#load-value-from-memory-to-register) |
| load_address_to_reg | [load address to register](#load-address-to-register) |
| add_assign | [add assign template](#add-assign-template) |
| mul_assign | [mul assign template](#mul-assign-template) |
| div_assign | [div assign template](#div-assign-template) |
| sub_assign | [sub assign template](#sub-assign-template) |
| save_value_to_address | [save value to address](#save-value-to-address) |
| cast_to_int | [cast to int](#cast-to-int) |
| convert_to_f64 | [convert to float64](#convert-to-float64) |
| add | [add operation](#add-operation) |
| sub | [sub operation](#sub-operation) |
| mul | [mul operation](#mul-operation) |
| div | [div operation](#div-operation) |
| save_local_variable_value | [save local variable value](#save-local-variable-value) |

## load const to a0

load const to regsiter a0

- output: ft0
- impacted_regs: t0

```asm
    .text
        la t0, labelx
        flw ft0, 0(t0)
    .data
        labelx: .float 144
```

## load value from memory to register

load value from memory to register using frame pointer (fp) and offset

- output: ft0

```asm
    .text
    flw ft0, 144(fp)

```

## load address to register

load float value from t0's address to ft0 register

- input: t0
- output: ft0

```asm
    .text 
    flw ft0, 0(t0)
```

## load id to t1

load id's value as uint to register ft1

- output: ft1
- impacted_regs: t0

```asm
    .text
    __label_load_id_to_t1:
    la t0, labelx
    flw ft1, 0(t0)
```

## add assign template

handle the add assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to ft2 already

- input: t1, ft2
- output: ft0

```asm
    .text
    __label__add_assign_template:
    flw ft0, 0(t1)
    fadd.s ft0, ft0, ft2
    fsw ft0, 0(t1)
```

## mul assign template

handle the mul assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to ft2 already

- input: t1, ft2
- output: ft0

```asm
    .text
    __label__mul_assign_template:
    flw ft0, 0(t1)
    fmul.s ft0, ft0, ft2
    fsw ft0, 0(t1)
```

## div assign template

handle the div assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to ft2

- input: t1, ft2
- output: ft0

```asm
    .text
    __label__div_assign_template:
    flw ft0, 0(t1)
    fdiv.s ft0, ft0, ft2
    fsw ft0, 0(t1)
```

## sub assign template

handle the sub assgin operator, the t1 is the address of the j, and the value of i (or constant 2) is loaded to ft2

- input: t1, ft2
- output: ft0

```asm
    .text
    __label__sub_assign_template:
    flw ft0, 0(t1)
    fsub.s ft0, ft0, ft2
    fsw ft0, 0(t1)
```

## save value to address

- input : t1, ft0
- output: ft0

save long to a address which is stored in a register t1 already

```asm
    .text 
    fsw ft0, 0(t1)
```

## cast to int

- input : ft0
- output: t0

convert value in ft0 to t0

```asm
    fcvt.w.s t0, ft0

```

## add operation

this template also used for float32 add

- input: ft0, ft1
- output: ft2

```asm
    .text 
    ___label_add_operation:
    fadd.s ft2, ft0, ft1
```

## sub operation

this template also used for float32 sub

- input: ft0, ft1
- output: ft2

```asm
    .text 
    ___label_add_operation:
    fsub.s ft2, ft0, ft1
```

## mul operation

this template also used for float32 mul

- input: ft0, ft1
- output: ft2

```asm
    .text 
    ___label_add_operation:
    fmul.s ft2, ft0, ft1
```

## div operation

this template also used for float32 div

- input: ft0, ft1
- output: ft2

```asm
    .text 
    ___label_add_operation:
    fdiv.s ft2, ft0, ft1
```

## convert to float64

this template is to convert float32 to float64

- input: ft0
- output: ft1

```asm
    fcvt.d.s ft1, ft0

```

## save local variable value

this code snippet is to save local variable value to stack. fp is the stack pointer and 144 will replaced with actual stack offset

- input: ft0

```asm
    .text
    fsw ft0, 144(fp)
```
