;redcode-94
;name Iron Gate
;author Wayne Sheppard
;strategy cmp scanner-SPL/JMP-Gate
;assert CORESIZE==8000

dist 	equ 73		; Old scan distance = 98
scan 	equ dist*2     
     
     	add 	off,	@x
loc  	cmp 	dist-1,	-1
     	slt 	#14,	@x
     	djn 	-3,	<7000
     	mov 	j,	@loc
x    	mov 	s,	<loc
     	sub 	new,	@x
     	jmn 	loc,	loc-1
s    	spl 	#0,	<1-dist
     	mov 	2,	<-2
j    	jmp 	-1
new  	dat 	<0-dist,<0-dist-1
off  	dat 	<scan,	<scan
end
