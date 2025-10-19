    .section .text.entry
    .globl _start
_start:
    /* OpenSBI: a0 = hartid, a1 = fdt (physical), MMU off. Everything is pc-relative until paging is on. */
    la sp, boot_stack_top
    call boot_enable_paging
1:  wfi
    j 1b

    .globl kernel_entry
kernel_entry:
    la sp, boot_stack_top
    la t0, _bss
    la t1, _ebss
2:  bgeu t0, t1, 3f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 2b
3:  call kmain
4:  wfi
    j 4b

    .section .bss
    .align 4
    .space 16384
    .globl boot_stack_top
boot_stack_top:

    .text
    .globl ctx_switch
ctx_switch:
    sd ra, 0(a0)
    sd sp, 8(a0)
    sd s0, 16(a0)
    sd s1, 24(a0)
    sd s2, 32(a0)
    sd s3, 40(a0)
    sd s4, 48(a0)
    sd s5, 56(a0)
    sd s6, 64(a0)
    sd s7, 72(a0)
    sd s8, 80(a0)
    sd s9, 88(a0)
    sd s10, 96(a0)
    sd s11, 104(a0)
    ld ra, 0(a1)
    ld sp, 8(a1)
    ld s0, 16(a1)
    ld s1, 24(a1)
    ld s2, 32(a1)
    ld s3, 40(a1)
    ld s4, 48(a1)
    ld s5, 56(a1)
    ld s6, 64(a1)
    ld s7, 72(a1)
    ld s8, 80(a1)
    ld s9, 88(a1)
    ld s10, 96(a1)
    ld s11, 104(a1)
    ret

    .align 4
    .globl trap_entry
trap_entry:
    addi sp, sp, -288
    .irp n,1,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
    sd x\n, \n*8(sp)
    .endr
    addi t0, sp, 288
    sd t0, 16(sp)
    csrr t0, sepc
    sd t0, 256(sp)
    csrr t0, sstatus
    sd t0, 264(sp)
    csrr t0, scause
    sd t0, 272(sp)
    csrr t0, stval
    sd t0, 280(sp)
    mv a0, sp
    call trap_handler
    ld t0, 256(sp)
    csrw sepc, t0
    ld t0, 264(sp)
    csrw sstatus, t0
    .irp n,1,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
    ld x\n, \n*8(sp)
    .endr
    addi sp, sp, 288
    sret
