.syntax unified
.thumb

.extern __cortexos_switch_context

.global PendSV
.type PendSV, %function
.thumb_func
PendSV:
    mrs     r0, psp
    isb
    stmdb   r0!, {r4-r11}
    mov     r2, r0
    stmdb   sp!, {r2, r3, r12, lr}
    bl      __cortexos_switch_context
    ldmia   sp!, {r2, r3, r12, lr}
    cbnz    r0, 1f
    mov     r0, r2
1:
    ldmia   r0!, {r4-r11}
    msr     psp, r0
    isb
    bx      lr
