;name Tornado
;author Beppe Bezzi
;strategy Fast 60% c bomber
;assert CORESIZE == 8000
MOV.I  $7, *2
MOV.I  $6, @1
MOV.I  $97, @192
ADD.F  $2, $-1
DJN.B  $-4, #533
SPL.B  #285, #285
MOV.I  $1, >-7
DAT.F  $95, $95
