;redcode-94
;assert CORESIZE == 8000

;author Kurt Franke
;name Stepping Stone
;kill Stepping Stone

;strategy Quick scan -> Vampire (version 9h)

;components (to help Planar) :
; quickscan, boot, vamp, scan, stun, clear, pspace(brainwash)

;; - - - - - - - - - - - boot parameters - - - - - - - - - - - - - - - -

				;; reference address for boot distances
REF		equ	(vamp-3)

				;; first define the places things will go
BVAMP		equ	(3+282*10+REF)
BSTEP		equ	(6+295*10+REF)
BJUMP		equ	(1+298*10+REF)
BCLEAR		equ	(4+367*10+REF)
BPIT		equ	(5+67*10+REF)
		;;	 ^
		;;	 +---- 	don't change this column of numbers
		;; 		they are magic to avoid self-bombing

				;; now define constants for use in the code
				;; (these are to be used when refering to a
				;;  line within that block of code)
CVAMP		equ	(BVAMP-vamp)
CCLEAR		equ	(BCLEAR-dbomb2+space)
CTARGET		equ	(BVAMP-vamp+target)
ADJUST		equ	(CCLEAR-CTARGET)

FIRST		equ	0
INC		equ	110	;; mod 10 step for mod 5 vamp/scan
COUNT		equ	8*5

;; - - - - - - - - - - - the vampire code - - - - - - - - - - - - - - -

vamp	add.f	$vamp+6,	$(BJUMP-CVAMP)	;; incr will be at vamp+6
	mov.i	@0,		@(BJUMP-CVAMP)
	jmz.a	*target,	*(BJUMP-CVAMP)
	mov.i	@0,		*(BJUMP-CVAMP)	;; a-field needs to be 0
target	djn	vamp,		#COUNT
	jmp	(CCLEAR-CVAMP)

step	jmp	@0,		BPIT-BSTEP	;; a-field needs to be 0

jbomb	jmp	@BSTEP-BJUMP-FIRST,	$FIRST	;; indirect jump to pit

CPTR		equ	(dbomb2-1)

dbomb2	dat	<2667, <2*2667+1
dbomb	dat	<-15, #10
space	spl	#0, #10				;; a-field needs to be 0
clear	mov	@cloop, >CPTR
	mov	@cloop, >CPTR
cloop	djn.b	$clear, {space

incr	dat	$-INC,		$INC

pit	mov.a	#ADJUST,	$CTARGET-BPIT
	spl	#0,		$0		;; a-field needs to be 0
	mov	2,		>1
	stp	#0,		$BCLEAR-BPIT+36

;; - - - - - - - - - - - - - boot code - - - - - - - - - - - - - - - - - -

pboot	mov	$pit+3, BPIT+3
	for	3
	  mov   {pboot, <pboot
	rof

cboot	mov	$dbomb2+5, BCLEAR+5
	for	5
	  mov	{cboot, <cboot
	rof

	mov	$step, BSTEP

	mov	$jbomb, BJUMP

	mov	$incr, BVAMP+6
vboot	mov	$vamp+5, BVAMP+5
	for	5
	  mov	{vboot, <vboot
	rof

	mov	$0, $cboot

	spl	@vboot

	mov	$0, $vboot			;; erase important pointers

	dat	$0, $0

;; - - - - - - - - - - - - - quick scan - - - - - - - - - - - - - - - - - -

QINC		equ	6
QCOUNT		equ	7

qjump	jmp	$qpit-found, #0
qstep1	jmp	*QINC, #-2*QINC
qstep2	jmp	*-2*QINC, #-QINC

start       ;'94 scan (from FAQ)
s1	for 4
	  sne.x	start+400*(s1+12), start+400*(s1+12)+100
	  seq.x	start+400*(s1+12)+200, start+400*(s1+12)+300
	  mov	#start+400*(s1+12)-found, $found
	rof
	jmn	which, $found

s2	for 4
	  sne.x	start+400*(s2+6), start+400*(s2+6)+100
	  seq.x	start+400*(s2+6)+200, start+400*(s2+6)+300
	  mov	#start+400*(s2+6)-found-100, $found
	rof
	jmn	which, $found

s3	for 4
	  sne.x	start+400*(s3+2), start+400*(s3+2)+100
	  seq.x	start+400*(s3+2)+200, start+400*(s3+2)+300
	  mov	#start+400*(s3+2)-found-100, $found
	rof

	jmz	pboot, $found
	add	#100, $found
which	jmz.f	-1, @found
	sub.ba	$found, $qjump
	mov	$qjump, @found
	mov.ba	$found, $found

qloop	sub.f	$qstep1, $found
	mov	$qstep2, @found
	mov	$qstep1, *found
found	mov	$0, @0
	djn	qloop, #QCOUNT
launch	jmp	pboot, #space

qpit	mov.ba	$launch, $launch
qerase	spl	#0, #start
	mov	$10, >qerase
	jmp	BPIT

;; - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

	end	start
