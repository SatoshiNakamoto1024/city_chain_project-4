.text	


.globl	RC4
.def	RC4;	.scl 2;	.type 32;	.endef
.p2align	4
RC4:
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%rax
.LSEH_begin_RC4:
	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx


.byte	243,15,30,250
	orq	%rsi,%rsi
	jne	.Lentry
	movq	8(%rsp),%rdi
	movq	16(%rsp),%rsi
	.byte	0xf3,0xc3
.Lentry:
	pushq	%rbx

	pushq	%r12

	pushq	%r13

.Lprologue:
	movq	%rsi,%r11
	movq	%rdx,%r12
	movq	%rcx,%r13
	xorq	%r10,%r10
	xorq	%rcx,%rcx

	leaq	8(%rdi),%rdi
	movb	-8(%rdi),%r10b
	movb	-4(%rdi),%cl
	cmpl	$-1,256(%rdi)
	je	.LRC4_CHAR
	movl	OPENSSL_ia32cap_P(%rip),%r8d
	xorq	%rbx,%rbx
	incb	%r10b
	subq	%r10,%rbx
	subq	%r12,%r13
	movl	(%rdi,%r10,4),%eax
	testq	$-16,%r11
	jz	.Lloop1
	btl	$30,%r8d
	jc	.Lintel
	andq	$7,%rbx
	leaq	1(%r10),%rsi
	jz	.Loop8
	subq	%rbx,%r11
.Loop8_warmup:
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	movl	%edx,(%rdi,%r10,4)
	addb	%dl,%al
	incb	%r10b
	movl	(%rdi,%rax,4),%edx
	movl	(%rdi,%r10,4),%eax
	xorb	(%r12),%dl
	movb	%dl,(%r12,%r13,1)
	leaq	1(%r12),%r12
	decq	%rbx
	jnz	.Loop8_warmup

	leaq	1(%r10),%rsi
	jmp	.Loop8
.p2align	4
.Loop8:
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	movl	0(%rdi,%rsi,4),%ebx
	rorq	$8,%r8
	movl	%edx,0(%rdi,%r10,4)
	addb	%al,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	%bl,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	movl	4(%rdi,%rsi,4),%eax
	rorq	$8,%r8
	movl	%edx,4(%rdi,%r10,4)
	addb	%bl,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	movl	8(%rdi,%rsi,4),%ebx
	rorq	$8,%r8
	movl	%edx,8(%rdi,%r10,4)
	addb	%al,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	%bl,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	movl	12(%rdi,%rsi,4),%eax
	rorq	$8,%r8
	movl	%edx,12(%rdi,%r10,4)
	addb	%bl,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	movl	16(%rdi,%rsi,4),%ebx
	rorq	$8,%r8
	movl	%edx,16(%rdi,%r10,4)
	addb	%al,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	%bl,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	movl	20(%rdi,%rsi,4),%eax
	rorq	$8,%r8
	movl	%edx,20(%rdi,%r10,4)
	addb	%bl,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	movl	24(%rdi,%rsi,4),%ebx
	rorq	$8,%r8
	movl	%edx,24(%rdi,%r10,4)
	addb	%al,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	$8,%sil
	addb	%bl,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	movl	-4(%rdi,%rsi,4),%eax
	rorq	$8,%r8
	movl	%edx,28(%rdi,%r10,4)
	addb	%bl,%dl
	movb	(%rdi,%rdx,4),%r8b
	addb	$8,%r10b
	rorq	$8,%r8
	subq	$8,%r11

	xorq	(%r12),%r8
	movq	%r8,(%r12,%r13,1)
	leaq	8(%r12),%r12

	testq	$-8,%r11
	jnz	.Loop8
	cmpq	$0,%r11
	jne	.Lloop1
	jmp	.Lexit

.p2align	4
.Lintel:
	testq	$-32,%r11
	jz	.Lloop1
	andq	$15,%rbx
	jz	.Loop16_is_hot
	subq	%rbx,%r11
.Loop16_warmup:
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	movl	%edx,(%rdi,%r10,4)
	addb	%dl,%al
	incb	%r10b
	movl	(%rdi,%rax,4),%edx
	movl	(%rdi,%r10,4),%eax
	xorb	(%r12),%dl
	movb	%dl,(%r12,%r13,1)
	leaq	1(%r12),%r12
	decq	%rbx
	jnz	.Loop16_warmup

	movq	%rcx,%rbx
	xorq	%rcx,%rcx
	movb	%bl,%cl

.Loop16_is_hot:
	leaq	(%rdi,%r10,4),%rsi
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	pxor	%xmm0,%xmm0
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	4(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,0(%rsi)
	addb	%bl,%cl
	pinsrw	$0,(%rdi,%rax,4),%xmm0
	jmp	.Loop16_enter
.p2align	4
.Loop16:
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	pxor	%xmm0,%xmm2
	psllq	$8,%xmm1
	pxor	%xmm0,%xmm0
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	4(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,0(%rsi)
	pxor	%xmm1,%xmm2
	addb	%bl,%cl
	pinsrw	$0,(%rdi,%rax,4),%xmm0
	movdqu	%xmm2,(%r12,%r13,1)
	leaq	16(%r12),%r12
.Loop16_enter:
	movl	(%rdi,%rcx,4),%edx
	pxor	%xmm1,%xmm1
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movl	8(%rsi),%eax
	movzbl	%bl,%ebx
	movl	%edx,4(%rsi)
	addb	%al,%cl
	pinsrw	$0,(%rdi,%rbx,4),%xmm1
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	12(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,8(%rsi)
	addb	%bl,%cl
	pinsrw	$1,(%rdi,%rax,4),%xmm0
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movl	16(%rsi),%eax
	movzbl	%bl,%ebx
	movl	%edx,12(%rsi)
	addb	%al,%cl
	pinsrw	$1,(%rdi,%rbx,4),%xmm1
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	20(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,16(%rsi)
	addb	%bl,%cl
	pinsrw	$2,(%rdi,%rax,4),%xmm0
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movl	24(%rsi),%eax
	movzbl	%bl,%ebx
	movl	%edx,20(%rsi)
	addb	%al,%cl
	pinsrw	$2,(%rdi,%rbx,4),%xmm1
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	28(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,24(%rsi)
	addb	%bl,%cl
	pinsrw	$3,(%rdi,%rax,4),%xmm0
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movl	32(%rsi),%eax
	movzbl	%bl,%ebx
	movl	%edx,28(%rsi)
	addb	%al,%cl
	pinsrw	$3,(%rdi,%rbx,4),%xmm1
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	36(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,32(%rsi)
	addb	%bl,%cl
	pinsrw	$4,(%rdi,%rax,4),%xmm0
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movl	40(%rsi),%eax
	movzbl	%bl,%ebx
	movl	%edx,36(%rsi)
	addb	%al,%cl
	pinsrw	$4,(%rdi,%rbx,4),%xmm1
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	44(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,40(%rsi)
	addb	%bl,%cl
	pinsrw	$5,(%rdi,%rax,4),%xmm0
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movl	48(%rsi),%eax
	movzbl	%bl,%ebx
	movl	%edx,44(%rsi)
	addb	%al,%cl
	pinsrw	$5,(%rdi,%rbx,4),%xmm1
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	52(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,48(%rsi)
	addb	%bl,%cl
	pinsrw	$6,(%rdi,%rax,4),%xmm0
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movl	56(%rsi),%eax
	movzbl	%bl,%ebx
	movl	%edx,52(%rsi)
	addb	%al,%cl
	pinsrw	$6,(%rdi,%rbx,4),%xmm1
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	addb	%dl,%al
	movl	60(%rsi),%ebx
	movzbl	%al,%eax
	movl	%edx,56(%rsi)
	addb	%bl,%cl
	pinsrw	$7,(%rdi,%rax,4),%xmm0
	addb	$16,%r10b
	movdqu	(%r12),%xmm2
	movl	(%rdi,%rcx,4),%edx
	movl	%ebx,(%rdi,%rcx,4)
	addb	%dl,%bl
	movzbl	%bl,%ebx
	movl	%edx,60(%rsi)
	leaq	(%rdi,%r10,4),%rsi
	pinsrw	$7,(%rdi,%rbx,4),%xmm1
	movl	(%rsi),%eax
	movq	%rcx,%rbx
	xorq	%rcx,%rcx
	subq	$16,%r11
	movb	%bl,%cl
	testq	$-16,%r11
	jnz	.Loop16

	psllq	$8,%xmm1
	pxor	%xmm0,%xmm2
	pxor	%xmm1,%xmm2
	movdqu	%xmm2,(%r12,%r13,1)
	leaq	16(%r12),%r12

	cmpq	$0,%r11
	jne	.Lloop1
	jmp	.Lexit

.p2align	4
.Lloop1:
	addb	%al,%cl
	movl	(%rdi,%rcx,4),%edx
	movl	%eax,(%rdi,%rcx,4)
	movl	%edx,(%rdi,%r10,4)
	addb	%dl,%al
	incb	%r10b
	movl	(%rdi,%rax,4),%edx
	movl	(%rdi,%r10,4),%eax
	xorb	(%r12),%dl
	movb	%dl,(%r12,%r13,1)
	leaq	1(%r12),%r12
	decq	%r11
	jnz	.Lloop1
	jmp	.Lexit

.p2align	4
.LRC4_CHAR:
	addb	$1,%r10b
	movzbl	(%rdi,%r10,1),%eax
	testq	$-8,%r11
	jz	.Lcloop1
	jmp	.Lcloop8
.p2align	4
.Lcloop8:
	movl	(%r12),%r8d
	movl	4(%r12),%r9d
	addb	%al,%cl
	leaq	1(%r10),%rsi
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%sil,%esi
	movzbl	(%rdi,%rsi,1),%ebx
	movb	%al,(%rdi,%rcx,1)
	cmpq	%rsi,%rcx
	movb	%dl,(%rdi,%r10,1)
	jne	.Lcmov0
	movq	%rax,%rbx
.Lcmov0:
	addb	%al,%dl
	xorb	(%rdi,%rdx,1),%r8b
	rorl	$8,%r8d
	addb	%bl,%cl
	leaq	1(%rsi),%r10
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%r10b,%r10d
	movzbl	(%rdi,%r10,1),%eax
	movb	%bl,(%rdi,%rcx,1)
	cmpq	%r10,%rcx
	movb	%dl,(%rdi,%rsi,1)
	jne	.Lcmov1
	movq	%rbx,%rax
.Lcmov1:
	addb	%bl,%dl
	xorb	(%rdi,%rdx,1),%r8b
	rorl	$8,%r8d
	addb	%al,%cl
	leaq	1(%r10),%rsi
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%sil,%esi
	movzbl	(%rdi,%rsi,1),%ebx
	movb	%al,(%rdi,%rcx,1)
	cmpq	%rsi,%rcx
	movb	%dl,(%rdi,%r10,1)
	jne	.Lcmov2
	movq	%rax,%rbx
.Lcmov2:
	addb	%al,%dl
	xorb	(%rdi,%rdx,1),%r8b
	rorl	$8,%r8d
	addb	%bl,%cl
	leaq	1(%rsi),%r10
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%r10b,%r10d
	movzbl	(%rdi,%r10,1),%eax
	movb	%bl,(%rdi,%rcx,1)
	cmpq	%r10,%rcx
	movb	%dl,(%rdi,%rsi,1)
	jne	.Lcmov3
	movq	%rbx,%rax
.Lcmov3:
	addb	%bl,%dl
	xorb	(%rdi,%rdx,1),%r8b
	rorl	$8,%r8d
	addb	%al,%cl
	leaq	1(%r10),%rsi
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%sil,%esi
	movzbl	(%rdi,%rsi,1),%ebx
	movb	%al,(%rdi,%rcx,1)
	cmpq	%rsi,%rcx
	movb	%dl,(%rdi,%r10,1)
	jne	.Lcmov4
	movq	%rax,%rbx
.Lcmov4:
	addb	%al,%dl
	xorb	(%rdi,%rdx,1),%r9b
	rorl	$8,%r9d
	addb	%bl,%cl
	leaq	1(%rsi),%r10
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%r10b,%r10d
	movzbl	(%rdi,%r10,1),%eax
	movb	%bl,(%rdi,%rcx,1)
	cmpq	%r10,%rcx
	movb	%dl,(%rdi,%rsi,1)
	jne	.Lcmov5
	movq	%rbx,%rax
.Lcmov5:
	addb	%bl,%dl
	xorb	(%rdi,%rdx,1),%r9b
	rorl	$8,%r9d
	addb	%al,%cl
	leaq	1(%r10),%rsi
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%sil,%esi
	movzbl	(%rdi,%rsi,1),%ebx
	movb	%al,(%rdi,%rcx,1)
	cmpq	%rsi,%rcx
	movb	%dl,(%rdi,%r10,1)
	jne	.Lcmov6
	movq	%rax,%rbx
.Lcmov6:
	addb	%al,%dl
	xorb	(%rdi,%rdx,1),%r9b
	rorl	$8,%r9d
	addb	%bl,%cl
	leaq	1(%rsi),%r10
	movzbl	(%rdi,%rcx,1),%edx
	movzbl	%r10b,%r10d
	movzbl	(%rdi,%r10,1),%eax
	movb	%bl,(%rdi,%rcx,1)
	cmpq	%r10,%rcx
	movb	%dl,(%rdi,%rsi,1)
	jne	.Lcmov7
	movq	%rbx,%rax
.Lcmov7:
	addb	%bl,%dl
	xorb	(%rdi,%rdx,1),%r9b
	rorl	$8,%r9d
	leaq	-8(%r11),%r11
	movl	%r8d,(%r13)
	leaq	8(%r12),%r12
	movl	%r9d,4(%r13)
	leaq	8(%r13),%r13

	testq	$-8,%r11
	jnz	.Lcloop8
	cmpq	$0,%r11
	jne	.Lcloop1
	jmp	.Lexit
.p2align	4
.Lcloop1:
	addb	%al,%cl
	movzbl	%cl,%ecx
	movzbl	(%rdi,%rcx,1),%edx
	movb	%al,(%rdi,%rcx,1)
	movb	%dl,(%rdi,%r10,1)
	addb	%al,%dl
	addb	$1,%r10b
	movzbl	%dl,%edx
	movzbl	%r10b,%r10d
	movzbl	(%rdi,%rdx,1),%edx
	movzbl	(%rdi,%r10,1),%eax
	xorb	(%r12),%dl
	leaq	1(%r12),%r12
	movb	%dl,(%r13)
	leaq	1(%r13),%r13
	subq	$1,%r11
	jnz	.Lcloop1
	jmp	.Lexit

.p2align	4
.Lexit:
	subb	$1,%r10b
	movl	%r10d,-8(%rdi)
	movl	%ecx,-4(%rdi)

	movq	(%rsp),%r13

	movq	8(%rsp),%r12

	movq	16(%rsp),%rbx

	addq	$24,%rsp

.Lepilogue:
	movq	8(%rsp),%rdi
	movq	16(%rsp),%rsi
	.byte	0xf3,0xc3

.LSEH_end_RC4:
.globl	RC4_set_key
.def	RC4_set_key;	.scl 2;	.type 32;	.endef
.p2align	4
RC4_set_key:
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%rax
.LSEH_begin_RC4_set_key:
	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx


.byte	243,15,30,250
	leaq	8(%rdi),%rdi
	leaq	(%rdx,%rsi,1),%rdx
	negq	%rsi
	movq	%rsi,%rcx
	xorl	%eax,%eax
	xorq	%r9,%r9
	xorq	%r10,%r10
	xorq	%r11,%r11

	movl	OPENSSL_ia32cap_P(%rip),%r8d
	btl	$20,%r8d
	jc	.Lc1stloop
	jmp	.Lw1stloop

.p2align	4
.Lw1stloop:
	movl	%eax,(%rdi,%rax,4)
	addb	$1,%al
	jnc	.Lw1stloop

	xorq	%r9,%r9
	xorq	%r8,%r8
.p2align	4
.Lw2ndloop:
	movl	(%rdi,%r9,4),%r10d
	addb	(%rdx,%rsi,1),%r8b
	addb	%r10b,%r8b
	addq	$1,%rsi
	movl	(%rdi,%r8,4),%r11d
	cmovzq	%rcx,%rsi
	movl	%r10d,(%rdi,%r8,4)
	movl	%r11d,(%rdi,%r9,4)
	addb	$1,%r9b
	jnc	.Lw2ndloop
	jmp	.Lexit_key

.p2align	4
.Lc1stloop:
	movb	%al,(%rdi,%rax,1)
	addb	$1,%al
	jnc	.Lc1stloop

	xorq	%r9,%r9
	xorq	%r8,%r8
.p2align	4
.Lc2ndloop:
	movb	(%rdi,%r9,1),%r10b
	addb	(%rdx,%rsi,1),%r8b
	addb	%r10b,%r8b
	addq	$1,%rsi
	movb	(%rdi,%r8,1),%r11b
	jnz	.Lcnowrap
	movq	%rcx,%rsi
.Lcnowrap:
	movb	%r10b,(%rdi,%r8,1)
	movb	%r11b,(%rdi,%r9,1)
	addb	$1,%r9b
	jnc	.Lc2ndloop
	movl	$-1,256(%rdi)

.p2align	4
.Lexit_key:
	xorl	%eax,%eax
	movl	%eax,-8(%rdi)
	movl	%eax,-4(%rdi)
	movq	8(%rsp),%rdi
	movq	16(%rsp),%rsi
	.byte	0xf3,0xc3

.LSEH_end_RC4_set_key:

.globl	RC4_options
.def	RC4_options;	.scl 2;	.type 32;	.endef
.p2align	4
RC4_options:

.byte	243,15,30,250
	leaq	.Lopts(%rip),%rax
	movl	OPENSSL_ia32cap_P(%rip),%edx
	btl	$20,%edx
	jc	.L8xchar
	btl	$30,%edx
	jnc	.Ldone
	addq	$25,%rax
	.byte	0xf3,0xc3
.L8xchar:
	addq	$12,%rax
.Ldone:
	.byte	0xf3,0xc3

.p2align	6
.Lopts:
.byte	114,99,52,40,56,120,44,105,110,116,41,0
.byte	114,99,52,40,56,120,44,99,104,97,114,41,0
.byte	114,99,52,40,49,54,120,44,105,110,116,41,0
.byte	82,67,52,32,102,111,114,32,120,56,54,95,54,52,44,32,67,82,89,80,84,79,71,65,77,83,32,98,121,32,60,97,112,112,114,111,64,111,112,101,110,115,115,108,46,111,114,103,62,0
.p2align	6


.def	stream_se_handler;	.scl 3;	.type 32;	.endef
.p2align	4
stream_se_handler:
	pushq	%rsi
	pushq	%rdi
	pushq	%rbx
	pushq	%rbp
	pushq	%r12
	pushq	%r13
	pushq	%r14
	pushq	%r15
	pushfq
	subq	$64,%rsp

	movq	120(%r8),%rax
	movq	248(%r8),%rbx

	leaq	.Lprologue(%rip),%r10
	cmpq	%r10,%rbx
	jb	.Lin_prologue

	movq	152(%r8),%rax

	leaq	.Lepilogue(%rip),%r10
	cmpq	%r10,%rbx
	jae	.Lin_prologue

	leaq	24(%rax),%rax

	movq	-8(%rax),%rbx
	movq	-16(%rax),%r12
	movq	-24(%rax),%r13
	movq	%rbx,144(%r8)
	movq	%r12,216(%r8)
	movq	%r13,224(%r8)

.Lin_prologue:
	movq	8(%rax),%rdi
	movq	16(%rax),%rsi
	movq	%rax,152(%r8)
	movq	%rsi,168(%r8)
	movq	%rdi,176(%r8)

	jmp	.Lcommon_seh_exit


.def	key_se_handler;	.scl 3;	.type 32;	.endef
.p2align	4
key_se_handler:
	pushq	%rsi
	pushq	%rdi
	pushq	%rbx
	pushq	%rbp
	pushq	%r12
	pushq	%r13
	pushq	%r14
	pushq	%r15
	pushfq
	subq	$64,%rsp

	movq	152(%r8),%rax
	movq	8(%rax),%rdi
	movq	16(%rax),%rsi
	movq	%rsi,168(%r8)
	movq	%rdi,176(%r8)

.Lcommon_seh_exit:

	movq	40(%r9),%rdi
	movq	%r8,%rsi
	movl	$154,%ecx
.long	0xa548f3fc

	movq	%r9,%rsi
	xorq	%rcx,%rcx
	movq	8(%rsi),%rdx
	movq	0(%rsi),%r8
	movq	16(%rsi),%r9
	movq	40(%rsi),%r10
	leaq	56(%rsi),%r11
	leaq	24(%rsi),%r12
	movq	%r10,32(%rsp)
	movq	%r11,40(%rsp)
	movq	%r12,48(%rsp)
	movq	%rcx,56(%rsp)
	call	*__imp_RtlVirtualUnwind(%rip)

	movl	$1,%eax
	addq	$64,%rsp
	popfq
	popq	%r15
	popq	%r14
	popq	%r13
	popq	%r12
	popq	%rbp
	popq	%rbx
	popq	%rdi
	popq	%rsi
	.byte	0xf3,0xc3


.section	.pdata
.p2align	2
.rva	.LSEH_begin_RC4
.rva	.LSEH_end_RC4
.rva	.LSEH_info_RC4

.rva	.LSEH_begin_RC4_set_key
.rva	.LSEH_end_RC4_set_key
.rva	.LSEH_info_RC4_set_key

.section	.xdata
.p2align	3
.LSEH_info_RC4:
.byte	9,0,0,0
.rva	stream_se_handler
.LSEH_info_RC4_set_key:
.byte	9,0,0,0
.rva	key_se_handler
