;name He Scans Alone
;author P.Kline
;strategy 80% f-scanner switches from SPL to DAT carpet
;assert CORESIZE == 8000
ORG     25
DAT.F   $100,   $4096
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
MOV.I   $14,    <-6
MOV.I   >-7,    >-7
JMN.F   $-2,    >-8
SUB.X   #-12,   $-9
SNE.I   *-10,   @-10
SUB.X   *3,     @-2
JMN.F   $3,     @-12
JMZ.F   $-4,    *-13
MOV.X   @-5,    @-5
SLT.B   @-6,    #27
DJN.B   $-10,   @-7
DJN.B   *-3,    #13
JMP.B   *-4,    }-12
DAT.F   $0,     $0
SPL.B   #1,     {1
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
MOV.I   <-1214, {-1212
MOV.I   <-1212, {-1210
MOV.I   <-1210, {-1208
DJN.F   $-18,   <-1207
