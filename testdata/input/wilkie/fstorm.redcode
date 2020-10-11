;redcode quiet
;name Fire Storm v1.1
;author W. Mintardjo
;strategy Stone with anti-imp core clear
;assert CORESIZE==8000

boot    EQU Storm+4000
step    EQU 155
init    EQU step
away    EQU 18

        SPL 0, 1
        JMZ -1, 1
        JMP @-1, 1
        DAT #2, #1
        DAT #5, #1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        SPL -1, 1
        SPL 0, <1
        SPL -1, @1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        SPL 0, 1
        JMZ -1, 1
        JMP @-1, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        SPL -1, 1
        SPL 0, <1
        SPL -1, @1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        SPL 0, 1
        JMZ -1, 1
        JMP @-1, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        SPL -1, 1
        SPL 0, <1
        SPL -1, @1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        SPL 0, 1
        JMZ -1, 1
        JMP @-1, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        SPL -1, 1
        SPL 0, <1
        SPL -1, @1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        SPL 0, 1
        JMZ -1, 1
        JMP @-1, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        SPL -1, 1
        SPL 0, <1
        SPL -1, @1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1
        JMN -2, 1
        JMZ -5, 1
        DAT #2, #1
        DAT #5, #1

launch  MOV Storm+4, boot+4
        MOV Storm+3, <launch
        MOV Storm+2, <launch
        MOV Storm+1, <launch
        MOV Storm+0, <launch
        MOV Storm-1, <launch
        MOV core, away+boot
        MOV fire, boot+Storm-ptr-away
        JMP boot, <2000

ptr     SPL 0, <Storm-away
Storm   MOV <1-step, 2+step
        SUB Storm+away, -1
        JMP -2, <-2000
clear   MOV @Storm, <Storm+away
        DJN -1, <3975

core    DAT #step, #0-step
fire    DAT <Storm-ptr-away-1, #0

        END launch
