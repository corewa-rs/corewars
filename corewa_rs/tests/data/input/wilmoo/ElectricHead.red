;redcode-94
;name Electric Head
;author Anton Marsden
;strategy P-Warrior which uses the P^2 switcher (see CW 58)
;assert CORESIZE==8000
;kill Electric Head

;--------------
;THE CORE CLEAR
;--------------

gate1 EQU (bomb1-19)
gate2 EQU (bomb1-18)
bomb2 EQU (bomb1+22)
cca EQU (-4000) ; not the actual value

        dat     19      ,   500     ; 1: 1/1   
        dat     -4040   ,   4045    ; 2: 2/2
bomb1   spl     #bomb2-gate1 , 45   ;20: 5/4
        mov     *gate1  ,   >gate1  ; 1: 1/1
        mov     *bomb2+2,   >gate2  ; 2: 2/2
        djn.f   -1      ,   {gate2  ; 3: 3/3
        dat     1       ,   45      ; 2: 2/2
        spl     #-40    ,   45      ; 3: 3/3

ccb   mov bomb1-1,@ccd
      mov {-1,<ccd
      add #23,ccd
      spl 1,bomb1+4
      mov <ccb+3,<ccd
ccd   mov {ccb+1,{cca
      mov <ccb+3,<ccd
      djn @ccd,#2
      div.f #0,ccd

;--------
;MINI HSA
;--------

step EQU 9
ptr EQU (bomb-5)
away EQU (bomb1-4000) ; not the actual value

bomb: spl    #1,{1
kill: mov    bomb,<ptr
mptr: mov    >ptr,>ptr
      jmn.f  kill,>ptr
a:    add    #step+1,@mptr
scan: jmz.f  a,<ptr
      slt    @mptr,#btm-ptr+3
      djn    kill,@mptr
      djn    a,#16
btm:  jmp    a,{kill

boot: mov    btm,@dest
N FOR 8
      mov    btm-N,<dest
ROF
      spl    @dest,1
dest: mov    #250,@away
      mov    bomb,<dest
      div.f  #0,dest

FOR 6
      dat    0,0
ROF

dbomb dat    >-1,>1
in    dat    0,loss_table-state
p     spl    1,win_table-state
      spl    1,tie_table-state

;----------
;MINI PAPER
;----------

      spl    @0,1234 ; not the actual value
      mov    }-1,>-1
      mov    {-2,<1
      jmp    @0,>5678 ; not the actual value

PSTATE EQU 250 ; pspace location containing current state
STATES EQU 10  ; maximum number of states (for brainwash protection)

;NOTE: state values go from 0 to STATES-1

w0 EQU ccb   ; THE CORE CLEAR
w1 EQU boot  ; MINI HSA
w2 EQU p     ; MINI PAPER
w3 EQU cboot ; CARBONITE


think ldp.a  #0,in              ; get input value
load  ldp.a  #PSTATE,state      ; load old state
      mod.a  #STATES,state      ; brainwash protection
      add.ba *in,state          ; select correct state table
store stp.a  *state,load        ; store new state

win_table
state jmp    @0,w0              ; jump to warrior code

init_state
      spl    #0,w0
      spl    #0,w0
      spl    #3,w1
      spl    #3,w1
      spl    #4,w1
      spl    #6,w2
      spl    #7,w3
      spl    #7,w3
      spl    #8,w3

tie_table      
      spl    #1,w0
      spl    #2,w0
      spl    #3,w1
      spl    #3,w1
      spl    #4,w1
      spl    #5,w1
      spl    #0,w0
      spl    #7,w3
      spl    #8,w3
      spl    #9,w3

loss_table      
      spl    #1,w0
      spl    #2,w0
      spl    #3,w1
      spl    #4,w1
      spl    #5,w1
      spl    #6,w2
      spl    #7,w3
      spl    #8,w3
      spl    #9,w3
      spl    #3,w1

;-----------
;CARBONITE++
;-----------

caway EQU (-100) ; not the actual value

cboot mov tar+1,<cdest
      mov tar,<cdest
      mov tar-1,<cdest
cdest mov dbomb,*caway
      spl <cdest,<2000
      mov tar-2,@cdest
      div.f #0,cdest
      spl    #0,<-1151+3
      mov    197,tar-197*3500
tar   add.ab {0,}0    
      djn.f  -2,<-1151

END think