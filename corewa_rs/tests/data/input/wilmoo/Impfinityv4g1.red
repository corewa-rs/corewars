;redcode-94
;name Impfinity v4g1
;author Planar
;strategy boot,imp,stone,clear
;strategy   With self-priming pump and very short fuse.
;assert CORESIZE == 8000 
;kill Impfinity

istep	equ	1143
bstep01	equ	2214
bstep02	equ	3285
magic01	equ	(2)
magic02	equ	(-1)

trash	equ	(Z-285)
impoff	equ	(Z+437)
pmpof01	equ	(impoff+1*istep-577)
pmpof02	equ	(impoff+4*istep-496)
stnof01	equ	(impoff+3*istep-215)
stnof02	equ	(impoff+6*istep-158)

	org	boot
Z
boot	spl	boot02

i FOR 2

boot&i
j FOR 4
	mov.i	<psrc&i, <pdst&i
	mov.i	}psrc&i, }pdst&i
ROF
	mov.i	*psrc&i, *pdst&i
	mov.i	instr&i, impoff+(i-1)*istep
	spl	@pdst&i, >trash-15-i*2
	jmp	*pdst&i, >trash-i*2

psrc&i	dat	bomb&i, pend&i
pdst&i	dat	stnof&i, pmpof&i+pend&i-pump&i

point&i	equ	(pump&i-pmpof&i+impoff+(i-1)*istep)

pump&i	spl	#1, >prime&i
ptr&i	spl	pump&i-pmpof&i+impoff-istep-1, {335+i
	add.f	#istep+1, ptr&i
prime&i	mov.i	point&i, point&i-2
pend&i

instr&i	mov.i	#1, istep

bomb&i	dat	<2667, <5334
stone&i	spl	#stone&i+bstep&i+magic&i, {-650
	mov.i	bomb&i, }stone&i
	add.f	#bstep&i-1, stone&i
j&i	djn.f	stone&i, <i-50
send&i

j FOR i==1
k FOR 17
	mov.i	#1, 1
	mov.i	#1, @1
	spl	#1, 1
ROF
ROF
ROF
	end
