;redcode-94
;name The Fugitive
;author David Moore
;strategy qscan, silk, imps
;assert CORESIZE==8000

;---------
; q^2 scan
;---------

QB equ 1000  ; scan pattern
QS equ 500
QI equ 250

qscan seq qscan+QB+(QS*0), qscan+QB+(QS*0)+QI
      jmp q0, 0

      seq qscan+QB+(QS*3), qscan+QB+(QS*3)+QI
      jmp q1, 0
      seq qscan+QB+(QS*6), qscan+QB+(QS*6)+QI
      jmp q1, {q1
      seq qscan+QB+(QS*7), qscan+QB+(QS*7)+QI
      jmp q1, }q1

      seq qscan+QB+(QS*1), qscan+QB+(QS*1)+QI
      djn.a q2, >q1
      seq qscan+QB+(QS*2), qscan+QB+(QS*2)+QI
      jmp q2, >q1
      seq qscan+QB+(QS*4), qscan+QB+(QS*4)+QI
      jmp q2, {q2
      seq qscan+QB+(QS*5), qscan+QB+(QS*5)+QI
      jmp q2, 0
      seq qscan+QB+(QS*8), qscan+QB+(QS*8)+QI
      jmp q2, {q1
      seq qscan+QB+(QS*9), qscan+QB+(QS*9)+QI
      jmp q2, }q1
      seq qscan+QB+(QS*10), qscan+QB+(QS*10)+QI
      jmp q2, }q2

      jmp pboot, >393

;-----------
; q^2 bomber
;-----------

qb1   dat 23,  23
qb2   dat  1,  34

      dat  QS*1,  QS*6
tab   dat  QS*2,  QS*3
      dat  QS*7,  QS*7

q2    add.ab tab, fwd
q1    add.b  tab, @-1
q0    sne   @fwd, pboot-1
      add  #QI-2, @2
      add.ba fwd, fwd
      mov    qb2, *fwd
      mov    qb2, @fwd
fwd   mov     88, @qscan+QB+(QS*0)
      sub    qb1, @-3
      djn     -4, #5
      jmp  pboot,  0
 
for 48
 dat 0,0
rof

;---------------
; Silk with Imps
;---------------

impy equ 2667

c1 equ 3855
c2 equ 2355
c3 equ  831
c4 equ 4000

pboot spl  1, >7648        ; 8 processes
      spl  1, >7356
      spl  1, >7212

      mov <1, {1               ;optional
      spl paper+c4+8, paper+8  ;extra launcher

paper spl  @0, >c1
      mov }-1, >-1
      spl  @0, >c2+impy
      mov }-1, >-1
      spl  @0, >c2
      mov }-1, >-1

      mov.i #1, {1         ; anti-imp bombing

      mov.i #c3, impy      ; my imp (a-field is data)

end qscan
