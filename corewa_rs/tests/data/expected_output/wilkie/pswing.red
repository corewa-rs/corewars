;name Porch Swing
;author Randy Graham
;strategy Now 80% bomb/scan with djn-stream once-thru
;assert 1
ORG     8
DAT.F   #-9,    $20
DAT.F   #-10,   $-2
DAT.F   #-21,   $20
DAT.F   #-27,   $20
SPL.A   #-33,   $20
SPL.A   #-44,   $20
DAT.F   $0,     $0
SUB.F   $6,     $3
MOV.I   $8,     *2
MOV.I   $7,     @1
SNE.I   @58,    *23
DJN.B   $-4,    <-430
ADD.F   $-2,    $-11
SPL.A   #-47,   <-47
MOV.I   @1,     >-13
DJN.B   $-1,    {-10
DAT.F   <12,    <-12
