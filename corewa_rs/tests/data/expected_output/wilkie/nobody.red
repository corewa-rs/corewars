;name nobody special
;author Mike Nonemacher
;strategy core-trashing, anti-imping, etc.
;assert CORESIZE==8000
ORG     13
SPL.B   $1,     #0
SPL.B   $1,     #0
SPL.B   $1,     #0
SPL.B   @0,     }1800
MOV.I   }-1,    >-1
SPL.B   @0,     }3740
MOV.I   }-1,    >-1
MOV.I   $4,     >287
ADD.AB  #13,    $2086
MOV.I   {-4,    <1
JMP.B   @0,     >-1874
DAT.F   >2667,  >5334
DAT.F   $0,     $0
SPL.B   $-13,   #0
SPL.B   $1,     #0
SPL.B   $1,     #0
SPL.B   $1,     #0
SPL.B   @0,     }1800
MOV.I   }-1,    >-1
SPL.B   @0,     }3740
MOV.I   }-1,    >-1
MOV.I   $3,     >-5384
MOV.I   {-3,    <1
JMP.B   @0,     >-1874
DAT.F   >2667,  >5334
