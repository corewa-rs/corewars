;name Blue Funk 3
;author Steven Morrell
;strategy Fixed another in-memory/in-register bug
;assert CORESIZE==8000
ORG     6
DAT.F   <21,    <2667
SPL.B   #-3044, <3044
MOV.I   >-3044, $3045
ADD.F   $-2,    $-1
DJN.F   $-2,    <-53
DAT.F   #-7,    #0
MOV.I   $-1,    $-194
SPL.B   @0,     $-200
MOV.I   $-7,    >-1
MOV.I   $-7,    >-2
MOV.I   $-7,    >-3
MOV.I   $-7,    >-4
SPL.B   $6,     #0
SPL.B   $3,     #0
SPL.B   $6,     #0
JMP.B   >0,     $6
SPL.B   $-1,    #0
JMP.B   >0,     $5338
SPL.B   $-4,    #0
SPL.B   $-2,    #0
JMP.B   >0,     $2668
MOV.I   #3044,  $2667
