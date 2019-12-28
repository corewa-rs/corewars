preload
  mov 0, 1
dat   1  ,  2
mov 1, 0

jmp loop
jmp begin

begin:  add # 1, @ 2
        sub 3, 4
        mul 5, 6
        div 7, 8
        mod 9, 10
           jmz 0
        jmn  0
        djn  0
        cmp  0
        seq  0
        sne  0
        slt  0
        spl  0
        nop  0

loop
        ; TODO rewrite grammar to handle org, equ, end
        ; org  0
        ; equ  0
        ; end  0

        mov.a 1, 2
        mov.b 1, 2
        mov.ab 1, 2
        mov.ba 1, 2
        mov.f 1, 2
        mov.x 1, 2
        mov.i 1, 2

        jmp loop

        end foo bar baz

Some Junk Here mov.ba 1,2