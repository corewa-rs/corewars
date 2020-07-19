;redcode-94
;name nobody special
;author Mike Nonemacher
;assert CORESIZE==8000
;strategy Paper like TimeScape, but with lots more
;strategy core-trashing, anti-imping, etc.
;kill nobody

STEP1   EQU     1800
STEP2   EQU     3740
STEP3   EQU     STEP1-STEP2+2*8-OFF
OFF     EQU     -50
DIST    EQU     289

org     start

pl1     spl     1
        spl     1
        spl     1
p1      spl     @0,     }STEP1
        mov.i   }-1,    >-1
p12     spl     @0,     }STEP2
        mov.i   }-1,    >-1
        mov.i   4,      >p12+DIST
        add.ab  #13,    p12+STEP1+DIST
        mov.i   {-4,    <1
        jmp     @0,     >STEP3
        dat.f   >2667,  >5334
for     72
        dat.f   0,      0
rof
start   spl     pl1
        spl     1
        spl     1
        spl     1
p2      spl     @0,     }STEP1
        mov.i   }-1,    >-1
        spl     @0,     }STEP2
        mov.i   }-1,    >-1
        mov.i   3,      >OFF-5334
        mov.i   {-3,    <1
        jmp     @0,     >STEP3
        dat.f   >2667,  >5334
