;redcode-94
;name Marcia Trionfale 1.3
;author Beppe Bezzi
;contact bezzi@iol.it
;NSFCWT round 3
;strategy attemping a more aggressive behaviour
;assert CORESIZE == 8000
org start
A0      equ     3488
A1      equ     1860
A2      equ     3740
AA0     equ     3620
AA1     equ     1270
AA2     equ     -350
   
start
	spl     starta, <1000   ;activate body n. 2
	
	spl     1,      <300    ;\
	mov     -1,     0       ;-\ generate 12 
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
bomb1   dat     >1,     }1      ;anti clear bomb

for MAXLENGTH-CURLINE-16
	dat     0,0
rof

starta        
	spl     1,      <300    ;\
	mov     -1,     0       ;-\ generate 12 
	spl     1,      <600    ;-/ consecutive processes
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
