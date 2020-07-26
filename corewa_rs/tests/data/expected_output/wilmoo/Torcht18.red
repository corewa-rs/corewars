;name Torch t18
;author P.Kline
;strategy t18: use jmz not djn for spl/mov bombing!
ORG     0
MOV.I   $26,    *9
MOV.I   $14,    @8
SPL.B   $1,     $1
MOV.I   <8,     {8
MOV.I   <7,     {7
MOV.I   <6,     {6
MOV.I   <5,     {5
DJN.B   *3,     #1
MOV.I   $-6,    @2
DIV.F   #4317,  #4306
DIV.F   #4308,  #4346
DIV.F   #4314,  #14
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $-7,    $14
DAT.F   $0,     $0
DAT.F   $-7,    $14
SPL.B   #-55,   $-54
SUB.AB  #108,   $1
MOV.I   $6,     *90774
MOV.I   $-3,    @-1
JMZ.B   $-3,    #0
MOV.I   @1,     >-10
DJN.B   $-1,    {-6
DAT.F   $0,     $0
MOV.I   $55,    >55
DAT.F   $0,     $0
DAT.F   $1,     $1
DAT.F   $-1016, $984
DAT.F   $-1017, $983
