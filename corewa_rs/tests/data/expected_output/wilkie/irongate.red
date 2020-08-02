;name Iron Gate
;author Wayne Sheppard
;strategy cmp scanner-SPL/JMP-Gate
;assert CORESIZE==8000
ADD.F   $12,    @5
CMP.I   $72,    $-1
SLT.AB  #14,    @3
DJN.B   $-3,    <-1000
MOV.I   $6,     @-3
MOV.I   $3,     <-4
SUB.F   $5,     @-1
JMN.B   $-6,    $-7
SPL.B   #0,     <-72
MOV.I   $2,     <-2
JMP.B   $-1,    $0
DAT.F   <-73,   <-74
DAT.F   <146,   <146
