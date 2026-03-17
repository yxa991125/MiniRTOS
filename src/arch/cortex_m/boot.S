.syntax unified
.thumb

.global __cortexos_first_sp
.section .bss
.align 3                 /* 8-byte alignment */
__cortexos_first_sp:
    .word 0

.section .text
.align 2

.global __start_first_task
.type   __start_first_task, %function
__start_first_task:
    /* r0 = first task sp (software frame base: R4..R11) */
    ldr r1, =__cortexos_first_sp
    str r0, [r1]

    svc 0

1:  b 1b
.size __start_first_task, . - __start_first_task

/* Export both names to match cortex-m-rt expectations */
.global SVCall
.global SVCall_Handler
.type   SVCall, %function
.type   SVCall_Handler, %function

.thumb_func
SVCall_Handler:
SVCall:
    /* --- 兜底修正 MSP：从向量表首项读取初始 MSP --- */
    ldr r1, =0x08000000     /* 向量表基址在 Flash */
    ldr r1, [r1]            /* 取 vector[0] = 初始 MSP */
    msr msp, r1
    isb

    /* load first_sp */
    ldr r0, =__cortexos_first_sp
    ldr r0, [r0]

    /* PSP = first_sp */
    msr psp, r0

    /* Thread mode use PSP (set SPSEL), keep nPRIV as-is */
    mrs r0, control
    orr r0, r0, #2
    msr control, r0
    isb

    /* pop software-saved regs: R4-R11 */
    mrs r0, psp
    ldmia r0!, {r4-r11}
    msr psp, r0

    /* exception return to Thread mode using PSP */
    ldr r0, =0xFFFFFFFD
    mov lr, r0
    bx  lr

.size SVCall, . - SVCall
