;name Marcia Trionfale 1.3
;author Beppe Bezzi
;strategy attemping a more aggressive behaviour
;assert CORESIZE == 8000
ORG     0
SPL.B   $18,    <1000
SPL.B   $1,     <300
MOV.I   $-1,    $0
SPL.B   $1,     <500
SPL.B   $1,     <400
SPL.B   @0,     }3488
MOV.I   }-1,    >-1
MOV.I   $8,     >123
SPL.B   @0,     }1860
MOV.I   }-1,    >-1
MOV.I   $5,     >1001
MOV.I   $3,     }2042
MOV.I   {-4,    <1
JMP.B   @0,     >3740
DAT.F   >2667,  >5334
DAT.F   >1,     }1
DAT.F   $0,     $0
SPL.B   $1,     <300
MOV.I   $-1,    $0
SPL.B   $1,     <600
SPL.B   $1,     <400
SPL.B   @0,     }3620
MOV.I   }-1,    >-1
MOV.I   $9,     >113
SPL.B   @0,     }1270
MOV.I   }-1,    >-1
MOV.I   $5,     >1001
MOV.I   $5,     }2042
MOV.I   {-3,    <2
JMP.B   @1,     >-350
DAT.F   >2667,  >5334
DAT.F   >1,     }1
