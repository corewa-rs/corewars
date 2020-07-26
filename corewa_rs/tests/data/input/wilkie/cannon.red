;redcode-94 quiet
;name Cannonade
;kill Cannonade
;author P.Kline, Paul Kline, pk6811s@acad.drake.edu
;assert CORESIZE==8000
;strategy break gates, score points
;strategy stone and gate-busting imp-spiral(s)

impsize2 equ 2667
impsize1 equ 2668
offset1  equ 1833-impsize2
offset2  equ offset1+15
offset3  equ offset1+15+2
decst    equ 400-(start*57)
stinc    equ 190
start    mov imp1,imp1+offset2+5
         mov imp1,<start
         mov imp1,<start
         mov imp1,<start
         mov imp1,<start
         mov imp1,<start
         mov imp2,imp1+offset1+impsize2
         mov <copystf,copystf-701
         mov <copystf,<copystt
         mov <copystf,<copystt
         mov <copystf,<copystt
         spl @copystt,<decst
         mov <copystf,<copystt
         spl lnch1,<decst
         spl lnch3,<decst
         
lnch2    spl lnch2b,<decst
lnch2a   spl lnch2ab,<decst
lnch2aa  spl 2,<decst
         jmp imp1+offset2+(impsize1*0),<decst
         jmp imp1+offset2+(impsize1*1),<decst
lnch2ab  spl 2,<decst
         jmp imp1+offset2+(impsize1*2),<decst
         jmp imp1+offset2+(impsize1*3),<decst
lnch2b   spl lnch2bb,<decst
lnch2ba  spl 2,<decst
         jmp imp1+offset2+(impsize1*4),<decst
         jmp imp1+offset2+(impsize1*5),<decst
lnch2bb  spl 2,<decst
         jmp imp1+offset2+(impsize1*6),<decst
         jmp imp1+offset2+(impsize1*7),<decst
         
lnch3    spl lnch3b,<decst
lnch3a   spl lnch3ab,<decst
lnch3aa  spl 2,<decst
         jmp imp1+offset3+(impsize1*0),<decst
         jmp imp1+offset3+(impsize1*1),<decst
lnch3ab  spl 2,<decst
         jmp imp1+offset3+(impsize1*2),<decst
         jmp imp1+offset3+(impsize1*3),<decst
lnch3b   spl lnch3bb,<decst
lnch3ba  spl 2,<decst
         jmp imp1+offset3+(impsize1*4),<decst
         jmp imp1+offset3+(impsize1*5),<decst
lnch3bb  spl 2,<decst
         jmp imp1+offset3+(impsize1*6),<decst
         jmp imp1+offset3+(impsize1*7),<decst
         
lnch1    spl lnch1b,<decst
lnch1a   spl lnch1ab,<decst
lnch1aa  spl 2,<decst
         jmp imp1+offset1+(impsize2*1),<decst
         jmp imp1+offset1+(impsize2*2),<decst
lnch1ab  spl 2,<decst
         jmp imp1+offset1+(impsize2*3),<decst
         jmp imp1+offset1+(impsize2*4),<decst
lnch1b   spl lnch1bb,<decst
lnch1ba  spl 2,<decst
         jmp imp1+offset1+(impsize2*5),<decst
         jmp imp1+offset1+(impsize2*6),<decst
lnch1bb  spl 2,<decst
         jmp imp1+offset1+(impsize2*7),<decst
         jmp imp1+offset1+(impsize2*8),<decst
         
imp2     mov 0,impsize2
imp1     mov 0,impsize1

stone    mov   <stspl+5+(stinc*800),stspl
stspl    spl   -1,<2
stadd    add   3,stone
         djn   -2,<5141+1
inc      mov   stinc,<-stinc
copystf  dat #0
copystt  dat #copystf-702
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         mov 0,2667
         end start
