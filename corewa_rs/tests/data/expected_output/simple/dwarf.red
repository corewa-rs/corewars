;redcode
;name Dwarf
;author A. K. Dewdney
;version 94.1
;date April 29, 1993
;strategy Bombs every fourth instruction.
;assert CORESIZE % 4 == 0
ORG     1
DAT.F   #0,     #0
ADD.AB  #4,     $-1
MOV.AB  #0,     @-2
JMP.A   $-2,    #0
