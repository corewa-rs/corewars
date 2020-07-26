;redcode-94 verbose
;name  Blue Funk 3
;author Steven Morrell
;strategy  Fixed another in-memory/in-register bug
;assert CORESIZE==8000
;macro

step equ 3044

for 78
dat <imp,<2667
rof

emerald   SPL #-step,<step
stone     MOV >-step,step+1
          ADD emerald,stone
cnt       DJN.f  stone,<emerald-50

cc   dat #-7

boot mov cc,out-200+5
out  spl @0,out-200
     mov emerald,>out
     mov emerald+1,>out
     mov emerald+2,>out
     mov emerald+3,>out
     spl i
     spl i31
i12  spl imp2
imp1 jmp >0,imp
i31  spl imp1
imp3 jmp >0,imp+5334
i    spl i12
     spl imp3
imp2 jmp >0,imp+2667

imp  mov.i #3044,2667

end boot
