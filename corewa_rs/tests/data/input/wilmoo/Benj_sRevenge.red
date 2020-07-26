;redcode-94 test
;name Benj's Revenge 1.0
;author Robert Macrae
;assert CORESIZE==8000
;strategy Q^2 (new), and Suicidal Paper

; Name obvious to those who know my son :-)
; (oops -- classic "Name" bug, but I like the new one too:)

        ORG start

; History
; 2.1   reverts to Probe attack.
; 2.6   puts gap after paper, before bomber.
; 2.9   2.6 is best yet, so drop to 44 scans.
; Time to stop fiddling!

; Q2NEW 
;       See q2table.pas for the code to get the table values.
;       These [9,5,13,1,2,17,16] give up to 28 distinct targets
;       with the greatest at only 34. Easier to use than Probe's,
;       as no references leak outside table.
;       Checked by exec k, exec k+1 and checking bomb fell on scan mark!
;       All the scans included as, depending on step, different ones
;       will need to be commented out to avoid self-scans.


; ----------------------------------------------
;-redcode-94 test
;-name Sad 1.0
;-author Robert Macrae
;-assert CORESIZE==8000
;-strategy Suicidal Paper

; 1.0  spends a lot of time bombing, often itself. This makes it
;      resistant to carpets and scanners, but it may lose to dwarves!
;      Constants straight from ccpaper.

len     EQU 9
fcp     EQU 3039
scp     EQU 2365
tcp     EQU 777

boot    spl 1,  >-3000    ; 7 processes replace 9 in CCPaper
        spl 1,  <-3200    ; for cost of extra Mov in launcher.
        mov -1, 0

frog    spl @0,         <fcp-len
        mov }-1,        >-1
        mov }-2,        >-2
        spl @0,         <scp
        mov }-1,        >-1
        spl @0,         <tcp
        mov }-1,        >-1
        mov 2,          <-fcp+len+1   ; Wipe uncle.
        jmp -1,         <-10
        dat <2667,      <2667*2
; ----------------------------------------------

for 21
   dat 0,0
rof

; ----------------------------------------------

QS      EQU 100              ; Illustration only!
QD      EQU 4000             ; Ditto.
QB      EQU (start+14*QS)    ; Ditto.
CR      EQU (fnd-which)
datz    EQU (table-3)

        dat     9*QS,  1*QS     ; can get 28 values from this table
table:  dat     5*QS,  2*QS     ;  
        dat    13*QS, 17*QS     ;  

P:      add.f  table,table  ; point into table. Nudge with }{>< and djn.f.
slow:   add.ab *P,fnd       ; adds an element A column (usually)       
fast:   add.b  @P,@-1       ; adds an element B column (usually)       

which:  sne.i  datz,@fnd    ; which half of scan hit?
        add.ab #QD,fnd

; ----------------------------------------------
; Original Probe attack

COUNT   EQU 6
GAP     EQU 15
REP     EQU 6

         mov.i  qbomb,@fnd
fnd:     mov.i  -GAP/2,@QB    ; picks up table as bomb...
         add.ba fnd,fnd
         mov.i  qbomb,*fnd
         add.f  qinc,fnd
         mov.i  qbomb,@fnd
         djn.b  -3,#REP
         jmp    boot,}-300

qbomb:   jmp    -200,GAP
qinc:    dat    GAP,-GAP


; ----------------------------------------------
;                  0/1 cycle 
; ----------------------------------------------

start:
        seq.i  QB+QS*0,QB+QS*0+QD
        jmp    which, 0                ; 0

        seq.i  QB+QS*2,QB+QS*2+QD
        jmp    fast, 0                 ; E
        seq.i  QB+QS*1,QB+QS*1+QD
        jmp    fast, <P                ; D
        seq.i  QB+QS*17,QB+QS*17+QD
        jmp    fast, >P                ; F

; ----------------------------------------------
;                   2 cycles
; ----------------------------------------------

        seq.i  QB+QS*7,QB+QS*7+QD
        jmp    slow, 0                 ; BE
        seq.i  QB+QS*6,QB+QS*6+QD
        jmp    slow, <P                ; BD
        seq.i  QB+QS*22,QB+QS*22+QD
        jmp    slow, >P                ; BF
        seq.i  QB+QS*11,QB+QS*11+QD
        jmp    slow, {P                ; AE
        seq.i  QB+QS*15,QB+QS*15+QD
        jmp    slow, }P                ; CE
        seq.i  QB+QS*10,QB+QS*10+QD
        djn.f  slow, P                 ; AD

        seq.i  QB+QS*5,QB+QS*5+QD
        jmp    >fast, 0                 ; B
        seq.i  QB+QS*9,QB+QS*9+QD
        jmp    >fast, {P                ; A
        seq.i  QB+QS*13,QB+QS*13+QD
        jmp    >fast, }P                ; C

; ----------------------------------------------
;                   3 cycles
; ----------------------------------------------

        seq.i  QB+QS*14,QB+QS*14+QD
        jmp    P, 0                     ; BBEE
        seq.i  QB+QS*8,QB+QS*8+QD
        jmp    P, <P                    ; BDE
;        seq.i  QB+QS*24,QB+QS*24+QD             ; KO to avoid self scan!
;        jmp    P, >P                    ; BEF   ; KO to avoid self scan!
        seq.i  QB+QS*12,QB+QS*12+QD
        jmp    P, {P                    ; ADE
        seq.i  QB+QS*32,QB+QS*32+QD
        jmp    P, }P                    ; CEF
        seq.i  QB+QS*20,QB+QS*20+QD
        djn.f  P, P                     ; AADD

        seq.i  QB+QS*4,QB+QS*4+QD
        jmp    }slow, 0                 ; EE
        seq.i  QB+QS*3,QB+QS*3+QD                  
        jmp    }slow, {P                ; DE
        seq.i  QB+QS*19,QB+QS*19+QD
        jmp    }slow, }P                ; FE
;        seq.i  QB+QS*2,QB+QS*2+QD               ; Duplicates a faster scan
;        djn.f  }slow, P                 ; DD    ; Duplicates a faster scan
         
;        seq.i  QB+QS*10,QB+QS*10+QD             ; Duplicates a faster scan
;        jmp    <fast, 0                ; BB     ; Duplicates a faster scan
        seq.i  QB+QS*18,QB+QS*18+QD
        djn.f  <fast, P                ; AA
         
        seq.i  QB+QS*36+CR,QB+QS*36+QD+CR         ; CR corrects for the gap
        jmp    }fast, 0                ; BBBBG    ; between fnd and which.
;        seq.i  QB+QS*26+CR,QB+QS*26+QD+CR
;        jmp    }fast, >P               ; BBG
;        seq.i  QB+QS*34+CR,QB+QS*34+QD+CR
;        jmp    }fast, {P               ; AAG
;        seq.i  QB+QS*42+CR,QB+QS*42+QD+CR
;        jmp    }fast, }P               ; CCG
;        seq.i  QB+QS*52+CR,QB+QS*52+QD+CR
;        djn.f  }fast, P                ; AAAAG
;        seq.i  QB+QS*16+CR,QB+QS*16+QD+CR
;        djn.f  }fast, }slow            ; G
         
        jmp    boot ; If you don't spot any, get sad ;-)

        end
