# CLang to ASM mapping

this file contains string snippet

| CLang | ASM |
| --- | --- |
| load_const_to_a0 | [load const to a0](#load-const-to-a0) |

## load const to a0

load const to regsiter a0

- output: t0
- impacted_regs: t0

```asm
    .text
        la t0, labelx
    .data
        labelx: .string "144"
```
