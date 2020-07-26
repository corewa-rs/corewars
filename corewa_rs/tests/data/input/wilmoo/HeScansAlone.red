;redcode-94
;name He Scans Alone
;kill He Scans Alone
;author P.Kline
;assert CORESIZE == 8000
;strategy 80% f-scanner switches from SPL to DAT carpet

tPtr      dat    100 ,4100-4             ; widely-spaced pointers
          dat    0,0
          dat    0,0
          dat    0,0
          dat    0,0
          dat    0,0
          
tWipe     mov    tSpl     ,<tPtr         ; positive wipe of opponent
          mov    >tPtr    ,>tPtr
          jmn.f  tWipe    ,>tPtr
          
tScan     sub.x  #-12     ,tPtr          ; increment and look
          sne    *tPtr    ,@tPtr
          sub.x  *pScan   ,@tScan        ; increment and look
          jmn.f  tSelf    ,@tPtr
          jmz.f  tScan    ,*tPtr
pScan     mov.x  @tScan   ,@tScan        ; swap pointers for attack
tSelf     slt.b  @tScan   ,#tEnd+4-tPtr  ; self-check
          djn    tWipe    ,@tScan        ;   go to attack
          djn    *pScan   ,#13           ; after 13 self-scans
          jmp    *pScan   ,}tWipe        ;   switch to dat-wiping
          dat    0,0
tSpl      spl    #1,{1
          dat    0,0
          dat    0,0
tEnd      dat 0,0
      for 61
          dat 0,0
      rof
tDecoy    equ    (tWipe+1-1196)
tStart    mov    <tDecoy+0,{tDecoy+2     ; make a quick-decoy
          mov    <tDecoy+3,{tDecoy+5     ; to foil one-shots
          mov    <tDecoy+6,{tDecoy+8     ; and the occasional q-scan
          djn.f  tScan+1  ,<tDecoy+10
          
          end    tStart
