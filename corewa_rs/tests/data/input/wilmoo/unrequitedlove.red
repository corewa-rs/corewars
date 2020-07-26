;redcode-94
;assert CORESIZE==8000
;name unrequited love
;author kafka
;strategy well, it's a departure from my normal title theme, but....
;strategy that's why I'm back doing corewar


org start
  
 dest0   equ   7889
 dest1   equ   3602
 dest2   equ   4095
 range   equ   1253
  
 paper
      spl   1,  <-300
      spl   1,  <-400
      spl   1,  <-500 
  
 silk    spl   @0,  {dest0
         mov.i }-1,>-1
 silk1   spl   @0,  <dest1
         mov.i }-1,>-1
         mov   bomba,}range
         mov   {silk1,<silk2
 silk2   jmp   @0,   >dest2
 bomba   dat   <2667, <5334

datz
i for 12
 dat 0,0
 rof

offset EQU (start+1000)
COUNT EQU 6

start:
N FOR COUNT
scan&N:
    seq.i  offset+400*N,offset+400*N+100
    jmp    kill&N+1
    seq.i  offset+400*N+200,offset+400*N+300
    jmp    kill&N

ROF
    jmp    paper

GAP EQU 15
REP EQU 6

datb:   dat  GAP,-GAP
dat200: dat  200,200
dat100: dat  100,100


N FOR COUNT
kill&N: add.f  dat200,pos&N      
      sne.i  datz,*pos&N
      add.f  dat100,pos&N
hit&N:  mov.i  datb,*pos&N
pos&N:  mov.i  offset+400*N,offset+400*N+GAP/2
      add.f  datb,pos&N
      djn.b  hit&N,#REP
      jmp    paper
ROF
