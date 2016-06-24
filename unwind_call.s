	.text
	.file	"unwind_call.ll"
	.globl	cleanup_fn
	.align	16, 0x90
	.type	cleanup_fn,@function
cleanup_fn:                             # @cleanup_fn
	.cfi_startproc
# BB#0:                                 # %entry
	retq
.Ltmp0:
	.size	cleanup_fn, .Ltmp0-cleanup_fn
	.cfi_endproc

	.globl	main
	.align	16, 0x90
	.type	main,@function
main:                                   # @main
	.cfi_startproc
# BB#0:                                 # %ifcont
	pushq	%rbx
.Ltmp1:
	.cfi_def_cfa_offset 16
	subq	$16, %rsp
.Ltmp2:
	.cfi_def_cfa_offset 32
.Ltmp3:
	.cfi_offset %rbx, -16
	movq	$32, 8(%rsp)
	movl	$32, %edi
	callq	malloc
	movq	%rax, %rbx
	movq	8(%rsp), %rdx
	xorl	%esi, %esi
	movq	%rbx, %rdi
	callq	memset
	movq	%rbx, (%rsp)
	movq	$0, (%rbx)
	movq	(%rsp), %rdi
	callq	_Unwind_RaiseException
	movl	$.L.str, %edi
	xorl	%eax, %eax
	callq	printf
	xorl	%eax, %eax
	addq	$16, %rsp
	popq	%rbx
	retq
.Ltmp4:
	.size	main, .Ltmp4-main
	.cfi_endproc

	.type	.L.str,@object          # @.str
	.section	.rodata.str1.1,"aMS",@progbits,1
.L.str:
	.asciz	"abhi"
	.size	.L.str, 5


	.section	".note.GNU-stack","",@progbits
