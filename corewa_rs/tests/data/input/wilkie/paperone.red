;redcode-94
;name Paperone
;author Beppe Bezzi
;strategy Silk replicator
;kill Paperone
;assert CORESIZE == 8000

start   spl     1,      <300    ;\
        spl     1,      <150    ;  generate 7 consecutive processes
        mov     -1,     0       ;/

silk    spl     3620,   #0      ;split to new copy
        mov.i   >-1,    }-1     ;copy self to new location

        mov.i   bomb,   >2005   ;linear bombing
        mov.i   bomb,   }2042   ;A-indirect bombing for anti-vamp

        add.a   #50,     silk    ;distance new copy   
        jmp     silk,   <silk   ;reset source pointer, make new copy
bomb    dat.f   >2667,  >5334   ;anti-imp bomb
