;name TimeScape (1.0)
;author J. Pohjalainen
;strategy v1.0: added more havoc in above code  (or I hope so!)
;assert CORESIZE==8000
ORG     9
SPL.B   @0,     }1800
MOV.I   }-1,    >-1
SPL.B   @0,     }3740
MOV.I   }-1,    >-1
MOV.I   {-1870, <1870
MOV.I   {-3,    <1
JMP.B   @0,     >-1922
DAT.F   $0,     $0
SPL.B   $1,     <-200
SPL.B   $1,     <-300
MOV.I   $-1,    $0
SPL.B   $-11,   <-400
SPL.B   @1,     }1800
MOV.I   }0,     >0
SPL.B   @1,     }3740
MOV.I   }0,     >0
MOV.I   <-1870, {1870
MOV.I   {-2,    <2
JMP.B   @1,     >-1922
