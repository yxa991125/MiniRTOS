.syntax unified
.thumb

.extern __cortexos_switch_context

.global PendSV
.type PendSV, %function
.thumb_func
PendSV:
    mrs     r0, psp
    stmdb   r0!, {r4-r11}
    mov     r4, lr
    bl      __cortexos_switch_context
    mov     lr, r4
    ldmia   r0!, {r4-r11}
    msr     psp, r0
    bx      lr
