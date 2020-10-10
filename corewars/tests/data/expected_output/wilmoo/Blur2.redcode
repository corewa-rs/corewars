;name Blur 2
;author Anton Marsden
;strategy Final version (for a while)
;assert CORESIZE==8000
ORG     2
MOV.I   $7,     >70
MOV.I   $5,     >-1
SEQ.I   $145,   $140
MOV.B   $-1,    @-2
ADD.F   $2,     $-2
DJN.B   $-4,    #800
SPL.I   #70,    #70
MOV.I   $2,     >-6
DJN.F   $-1,    >-7
DAT.F   <1,     #10
