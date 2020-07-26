;redcode-94

;kill       Scan Man

;name       Scan Man
;version    1.1.boot

;author     David van Dam

;date       15 May 1996

;strategy   0.66c scanner
;strategy
;strategy   v1.0       djn-stream protection
;strategy              spl/spl/dat/... core clear with djn-stream
;strategy   v1.1       Some gain on spl/incendairy bombers
;strategy   v1.1.boot  Boot + Decoy


;assert     CORESIZE == 8000


            org     boot


boot_loc    equ     4005-6000               ;990 2010 6000

disp        equ     1
range       equ     15
step        equ     30
offset      equ     (scanner+disp-range)

cc_magic    equ     -39

;1x Spl => -89
;2x Spl => -39


            dat     1       ,   2
            dat     3       ,   4
            dat     5       ,   6
            dat     7       ,   8

            dat     0       ,   0           ;<- Check
            dat     9       ,   10
            dat     11      ,   12
            dat     13      ,   14
            dat     15      ,   16

            dat     17      ,   18          ;-> Check
            dat     19      ,   20
            dat     21      ,   22
            dat     23      ,   24
            dat     25      ,   26

            dat     0       ,   0           ;<- Check
            dat     27      ,   28
            dat     29      ,   30
gate        dat     offset+range , offset+1
            dat     31      ,   32

            dat     33      ,   34          ;-> Check
            dat     35      ,   36
            dat     37      ,   38
            dat     39      ,   40
            dat     41      ,   42

            dat     17      ,   18          ;<- Check
            dat     43      ,   44
            dat     45      ,   46
            dat     47      ,   48

scanner     add.f   incr    ,   gate
cnt         sne.f   {gate   ,   >gate       ;-> Check
            jmz     scanner ,   #0

incr        spl.f   #-step+1,   <-step-1

cc          mov.i   *bombp  ,   >gate
            djn.f   cc      ,   {gate

            dat     33      ,   34          ;<- Check
            dat     49      ,   50
            dat     51      ,   52
            dat     53      ,   54
            dat     55      ,   56

            mov.i   *10     ,   @10         ;-> Check decoy
            dat     57      ,   58
            dat     59      ,   60
            dat     61      ,   62
            dat     63      ,   64

            sne.f   {-12    ,   <-12        ;<- Check decoy
            dat     65      ,   66
            dat     67      ,   68
            dat     69      ,   70
            dat     71      ,   72

            div.f   #1      ,   #1          ;->Check

            dat     #3      ,   #0          ; Anti-Djn stream
bombp       dat     #1      ,   #20
            spl     #cc_magic , #37


boot        mov     gate    ,   gate+boot_loc

            mov     *tgt1   ,   @tgt1       ;<- Check
            mov     {tgt1   ,   <tgt1
            mov     {tgt1   ,   <tgt1
            mov     {tgt1   ,   <tgt1
            mov     {tgt1   ,   <tgt1
            mov     {tgt1   ,   <tgt1       ;-> Check

            spl     @tgt1   ,   0           ;scanner start after 8c

            mov     }tgt2   ,   >tgt2
            mov     }tgt2   ,   >tgt2
            mov     }tgt2   ,   >tgt2       ;total copied after 13c

tgt1        div.f   #cc+1   ,   #gate+boot_loc+10+6     ;<- Check
tgt2        dat     #bombp-1,   #gate+boot_loc+10+6+17

            dat     73      ,   74
            dat     75      ,   76
            dat     77      ,   78

            dat     79      ,   80          ;-> Check
            dat     81      ,   82
            dat     83      ,   84
            dat     85      ,   86
            dat     87      ,   88

            mov.i   {5      ,   <5          ;<- Check decoy
            dat     89      ,   90
            dat     91      ,   92
            dat     93      ,   94
            dat     95      ,   96

            dat     97      ,   98          ;-> Check
            dat     99      ,   100
            dat     101     ,   102
            dat     103     ,   104
            dat     105     ,   106

            dat     79      ,   80          ;<- Check
            dat     107     ,   108
            dat     109     ,   110
            dat     111     ,   112
            dat     113     ,   114

            dat     0       ,   0           ;-> Check
            dat     115     ,   116
            dat     117     ,   118
            dat     119     ,   120
            dat     121     ,   122

            dat     97      ,   98          ;<- Check
            dat     123     ,   124
            dat     125     ,   126
            dat     127     ,   128
            dat     129     ,   130

            dat     0       ,   0           ;-> Check

            end
