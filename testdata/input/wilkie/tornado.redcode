;redcode-94
;name Tornado
;author Beppe Bezzi
;strategy the original one
;strategy Fast 60% c bomber
;assert CORESIZE == 8000

step    equ     95    
count   equ     533   
gate    equ     start-1

   
start   mov     bombd,  *stone
	mov     bombd,  @stone
stone   mov     step+2, @(2*step)+2         
	add     incr,   stone           
jump    djn.b   start,  #count
incr    spl     #3*step,#3*step        
clr     mov     bombd,  >gate
bombd   dat     step,   step
