;redcode-94
;name Jack in the box
;author Beppe Bezzi
;strategy Marcia Trionfale 1.3 with Tornado
;strategy popping up if confronted by a scanner
;kill Jack
;assert CORESIZE == 8000
org     think

PIN     0		;or something larger

_RES    equ     #0
_STR    equ     #111	;Not that obviously

step    equ     85         
count   equ     533
away    equ     4000	;+- 3000 locations

A0      equ     3488
A1      equ     1860
A2      equ     3740
AA0     equ     3620
AA1     equ     1270
AA2     equ     -350
   
marcia
	spl     starta, <1000   ;activate body n. 2
	
	spl     1,      <300    ;\
	spl     1,      <600    ;-\ generate 16 
	spl     1,      <500    ;-/ parallel processes
	spl     1,      <400    ;/

silk    spl     @0,     }A0     ;split 
	mov.i   }-1,    >-1     ;copy 
	mov.i   bomb1,  >123    ;bombing
silk2   spl     @0,     }A1     ;split
	mov.i   }-1,    >-1     ;copy
	mov.i   bomb1,  >1001   ;bombing
	mov.i   bomb ,  }2042   ;A-indirect bombing
	mov.i   {silk2, <silk3  ;copy
silk3   jmp     @0,     >A2     ;jmp new copy 
bomb    dat.f   >2667,  >5334   ;anti-imp bomb
bomb1   dat     >1,     }1      ;anti clear and djn stream bomb

for 12
	dat     0,0
rof

think
res     ldp.ab  _RES,   #0
str     ldp.a   _STR,   str1    ;load strategy in use
	sne.ab  #0,     res     ;check result
lost    add.a   #1,     str1    ;lost change 
	mod.a   #2,     str1    ;secure result
win     stp.ab  str1,   _STR
str1    jmp     @0,     tornado
	dat     0,      marcia


for 12
	dat     0,0
rof

tornado mov     bm,     djmp+away+31
	mov     bd,     djmp+away+32
	mov     *tptr,  @tptr
	mov     {tptr,  <tptr
	mov     {tptr,  <tptr
	mov     {tptr,  <tptr
	mov     {tptr,  <tptr
	mov     {tptr,  <tptr
	mov     {tptr,  <tptr
	mov     {tptr,  <tptr
	jmp     @tptr
tptr    dat     djmp,   away+djmp

bomber  mov     bd+30,  *stone
	mov     bm+30,  @stone
stone   mov     *step+2,*(2*step)+2         
	add     incr,   stone           
jump    djn.b   bomber, #count
incr    spl     #3*step,#3*step        
clr     mov     -12,    }bomber+1
djmp    jmp     clr,    <bomber-5

bm      mov     step,   1        
bd      dat     #step, #1

for 12
	dat     0,0
rof
starta        
	spl     1,      <300    ;\
	spl     1,      <600    ;-\ generate 16 
	spl     1,      <500    ;-/ consecutive processes
	spl     1,      <400    ;/

silka   spl     @0,     }AA0    ;split 
	mov.i   }-1,    >-1     ;copy 
	mov.i   bomba1, >113    ;bombing
silk2a  spl     @0,     }AA1    ;split 
	mov.i   }-1,    >-1     ;copy 
	mov.i   bomba,  >1001   ;bombing
	mov.i   bomba1, }2042   ;A-indirect bombing
	mov.i   {silk2a,<silk3a ;copy 
silk3a  jmp     @silk3a,>AA2    ;jmp new copy 
bomba   dat.f   >2667,  >5334   ;anti-imp bomb
bomba1  dat     >1,     }1      ;anti clear bomb


end
