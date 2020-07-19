;redcode-94
;name Thermite 1.0
;kill Phosphorus
;author Robert Macrae
;strategy  Quick-scan -> incendiary bomber.
;assert CORESIZE == 8000

; Since I don't launch phosphorus, vulnerable to carpet bombers. May
; pay to put it at start? I should make better use of DJN stream 
; (nascent). Either use <, or else start it somewhere which gets bombed
; by mov fairly quickly. What happens if I fall through early, due to
; DAT 1,1s? Should check this doesn't hurt...
 
SPC     equ 7700            ; (CORESIZE-MAXLENGTH-MINDISTANCE*2) 
STP1     equ 81              ; (SPC / (RAM/2) / 2)              
Lookat  equ look+237+8*(qscan-1)*STP1                        

; First scan at 237; last at -67?

traptr  dat     #0,     #trap
bite    jmp     @traptr,    0         ; Vampire bite. 

; Lots of pointers to these, so keep them away from trap!

look
qscan   for     6  
	sne.i   Lookat+0*STP1, Lookat+2*STP1
	seq.i   Lookat+4*STP1, Lookat+6*STP1
	mov.ab  #Lookat-bite-2*STP1, @bite
	rof

	jmn     test+1, bite   ; Save a few cycles

qscan   for     6  
	sne.i   Lookat+48*STP1, Lookat+50*STP1
	seq.i   Lookat+52*STP1, Lookat+54*STP1
	mov.ab  #Lookat-bite+46*STP1, @bite
	rof

	jmn     test+1, bite   ; Save a few cycles

qscan   for     6  
	sne.i   Lookat+1*STP1, Lookat+3*STP1
	seq.i   Lookat+5*STP1, Lookat+7*STP1
	mov.ab  #Lookat-bite-STP1, @bite
	rof

	jmn     test+1, bite   ; Save a few cycles

qscan   for     6   ; Should be 7 if I had space...  
	sne.i   Lookat+49*STP1, Lookat+51*STP1
	seq.i   Lookat+53*STP1, Lookat+55*STP1
	mov.ab  #Lookat-bite+47*STP1, @bite
	rof

; Intention is to place points evenly through the target area.

test    jmz.b   blind,  bite            ; if no address stored, no hit.
	add     #STP1*2, bite            ; Smaller than pyramid, as fast.
	jmz.f   -1,     @bite           ; find nonzero element.

	mov     spb,    @bite           ; Quick pre-bomb...

	add     #49,    bite            ; aim 51 past the hit
attack  sub.ba  bite,   bite            ; bite(b) contains target-bite
loop    mov     bite,   @bite           ; (a) contains the bite addr.
	add.f   step,   bite
	djn     loop,   #24             ; 6 spacing => 72 cycles...


; Incendiary bomber based on Phosphorus 1.0 (from Torch).

bstp     equ    155       ; Mod 5, as too big for mod 4 to miss!
gap      equ    15        ; Gap between mov and spl.
offset   equ    130       ; Chosen with step and gap to give long bombing run.
count    equ    1500

blind
spb      spl    #0,         <-gap+1    ; spl half of the incendiary
	 add    #bstp,      1                 
	 mov    spb,        @tgt-offset ; Gives longest run, given gap & step.
	 mov    mvb,        @-1                
tgt      djn.f  -3,         >300       ; gets bombed with spl to start clear
	 mov    ccb,        >spb-1     ; Uses copied mvb for CC.
	 djn.f  -1,         <spb-18    ; Aids clear.
mvb      mov    gap,        >gap       ; mov half of the incendiary
ccb      dat    0,          10         ; Core Clear.

; Bit worried about having trap so close to my code...
	
trap    spl     0,      >-200         ; Lackadaisical attempt at gates.     
	spl     -1,     >-200+2667    ; Each increments many times between
	jmp     -2,     >-200+2*2667  ; imp steps, but then the whole imp
				      ; moves! I only blow away rings...

step    dat     #6,     #-6           ; QS step size. Up from 5 for speed.

	end     look
