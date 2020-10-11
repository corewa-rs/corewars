;redcode-94
;name Porch Swing
;kill Porch Swing
;Author Randy Graham
;assert 1
;strategy Swing with a little wider range.
;strategy Now 80% bomb/scan with djn-stream once-thru

STEP    equ    12
EXTRA   equ    4
DJNOFF  equ    (-426-EXTRA)

        dat.f  #gate-10,  step-gate+5
gate    dat.f  #gate-10,  sneer-STEP+1
dat2    dat.f  #gate-20,  step-gate+5
dat1    dat.f  #gate-25,  step-gate+5
site2   spl.a  #gate-30,  step-gate+5
site    spl.a  #gate-40,  step-gate+5
for EXTRA
        dat.f  0,         0
rof

adder   sub.f  sweeper,   sneer
hithigh mov.i  step,      *sneer
hitlow  mov.i  step,      @sneer
sneer   sne.i  @gate+STEP*6-1-EXTRA, *gate+STEP*3-EXTRA  ;so we bomb step
looper  djn.b  adder,     <DJNOFF
setup   add.f  sneer,     gate
sweeper spl.a  #-STEP*4+1,<-STEP*4+1
mover   mov.i  @swinger,  >gate
swinger djn.b  mover,     {site
step    dat.f  <STEP,     <-STEP

end     hithigh
