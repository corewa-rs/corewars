;name Porch Swing
;author Randy Graham
;strategy Now 80% bomb/scan with djn-stream once-thru
;assert 1
ORG     11
DAT.F   #-9,    $23
DAT.F   #-10,   $1
DAT.F   #-21,   $23
DAT.F   #-27,   $23
SPL.A   #-33,   $23
SPL.A   #-44,   $23
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
DAT.F   $0,     $0
SUB.F   $6,     $3
MOV.I   $8,     *2
MOV.I   $7,     @1
SNE.I   @55,    *20
DJN.B   $-4,    <-430
ADD.F   $-2,    $-14
SPL.A   #-47,   <-47
MOV.I   @1,     >-16
DJN.B   $-1,    {-13
DAT.F   <12,    <-12
