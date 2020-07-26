;redcode-94 test
;name Newt
;author Ian Oversby
;strategy Q^2 -> Imp/Stone
;assert 1

gate1   equ     (init-7-dist)
pat     equ     3315
sval    equ     (ipos+5500)
ival    equ     (sval-2500)
dist    equ     3

impy    equ     (imp+sep)
sep     equ     1200
st      equ     2667

QB EQU (start+550)
QS EQU (QD*2)
QD EQU 100

GAP EQU 12
REP EQU 8
REP2 EQU 2

datz EQU (table-3)

  dat    10*QS, 2*QS ; can get 21 values from this table
table:   dat     4*QS, 1*QS ; and can also use the initial value
  dat    23*QS, 3*QS ; of fnd

qinc:    spl    #GAP,-GAP
tab:     add.a  table,table
slow:    add.a  @tab,fnd
fast:    add.ba *tab,@slow
which:   sne.i  datz,*fnd
  add.a  #QD,fnd
  mov.i  datone,*fnd
  add.ab fnd,fnd

fnd:    mov.i  QB,GAP/2
        add.f  qinc,fnd
 mov.i  datone,*fnd
 djn.b  fnd,#REP
 jmp    boot,}QS*13

start:
     ; WHICH
 seq.i  QB+QS*0,QB+QS*0+QD
 jmp    which,}QB+QS*0+QD/2

      ; FAST
  seq.i  QB+QS*1,QB+QS*1+QD
  jmp    fast,}QB+QS*1+QD/2

  seq.i  QB+QS*13,QB+QS*13+QD
  jmp    fast,{fast
  seq.i  QB+QS*2,QB+QS*2+QD
  jmp    fast,{tab
  seq.i  QB+QS*3,QB+QS*3+QD
  jmp    fast,}tab

     ; SLOW
  seq.i  QB+QS*4,QB+QS*4+QD
  jmp    >fast,}QB+QS*4+QD/2
  seq.i  QB+QS*5,QB+QS*5+QD
  jmp    slow,}QB+QS*5+QD/2

  seq.i  QB+QS*6,QB+QS*6+QD
  jmp    slow,{tab
  seq.i  QB+QS*7,QB+QS*7+QD
  jmp    slow,}tab
  seq.i  QB+QS*10,QB+QS*10+QD
  jmp    >fast,<tab
  seq.i  QB+QS*11,QB+QS*11+QD
  jmp    slow,<tab
  seq.i  QB+QS*12,QB+QS*12+QD
  djn.f  slow,tab
  seq.i  QB+QS*23,QB+QS*23+QD
  jmp    >fast,>tab
  seq.i  QB+QS*24,QB+QS*24+QD
  jmp    slow,>tab
  seq.i  QB+QS*17,QB+QS*17+QD
  jmp    slow,{fast

     ; TAB

  seq.i  QB+QS*8,QB+QS*8+QD
  jmp    <fast,}QB+QS*8+QD/2
  seq.i  QB+QS*9,QB+QS*9+QD
  jmp    tab,}QB+QS*9+QD/2

         seq.i  QB+QS*15,QB+QS*15+QD
  jmp    tab,<tab

  seq.i  QB+QS*16,QB+QS*16+QD
  jmp    tab,{tab
  seq.i  QB+QS*20,QB+QS*20+QD
  djn.f  <fast,tab

boot    MOV.I   cbomb,  @sptr
        MOV.I   <spos,  {sptr
        SPL.B   iboot,  <-300
for 6
        MOV.I   <spos,  {sptr
rof

        SPL.B   *sptr,  <-200

sptr    DIV.F   #init+sval,     #init+sval-7-3

cbomb   DAT.F   <5335,  #3+hit-gate1
; 2 DATS
init    SPL.B   #0,     <stone-pat
stone   SPL.B   #pat,   <-pat
loop    MOV.I   {0+pat, hit-pat
        ADD.F   stone,  loop
hit     DJN.F   loop,   <stone-pat
        MOV.I   init-dist, >gate1
last    DJN.F   -1,     >gate1

spos    DAT.F   $0,     $0

;;--------------------- Boot the imp/launch -------------------------

iboot   MOV.I   <ipos,  {iptr
for 4
        MOV.I   <ipos,  <iptr
rof
        SPL.B   @iptr,  <-300

iptr    DIV.F   #init+ival+sep+1,       #init+ival
datone  DAT.F   }300,   >200

spin    SPL.B   #st+1,  >prime
prime   MOV.I   impy,   impy
        ADD.F   spin,   jump
jump    JMP.B   impy-st-1, <-535
imp     MOV.I   #st,    *0

ipos    DAT.F   $0,     $0

for 10
        DAT.F   $0,     $0
rof

;;-------------------------------------------------------------------

 end start
