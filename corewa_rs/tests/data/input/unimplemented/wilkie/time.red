;Address jippo@clinet.fi
;redcode-94 verbose
;name TimeScape (1.0)
;author J. Pohjalainen
;assert CORESIZE==8000
;strategy I'm stuck with replicators! Here is _The Latest_ one!
;strategy \---------------------------\  ----------------------
;strategy / ts1  spl    @ts1,  }STEP1 /  Phoenix/Cell   warrior  
;strategy \      mov.i  }ts1,  >ts1   \  body,  6+ processes to 
;strategy / ts2  spl    @ts2,  }STEP2 /  keep That Thing alive,
;strategy \      mov.i  }ts2,  >ts2   \  two  of  them  working   
;strategy /      mov.i  {ts2,  <ts3   /  together  with  proper  
;strategy \ ts3  jmp    @ts3,  }STEP3 \  constants and you have
;strategy /___________________________/  found >>--> TimeScape!   
;strategy \T I M E   T O   E S C A P E\  ----------------------
;strategy v1.0: added more havoc in above code  (or I hope so!)
;kill TimeScape

TSTEP equ 1800
CSTEP equ 3740
NSTEP equ -1922
FSTEP equ 1870

tim1    spl     @tim1,          }TSTEP
        mov.i   }tim1,          >tim1
cel1    spl     @cel1,          }CSTEP
        mov.i   }cel1,          >cel1
        mov.i   {-FSTEP,        <FSTEP
        mov.i   {cel1,          <ncl1
ncl1    jmp     @ncl1,          >NSTEP

st for 82
        dat.f   0,              0
rof

warrior

        spl     1,              <-200
        spl     1,              <-300
        mov.i   -1,             0

        spl     tim1,           <-400

tim2    spl     @tim2,          }TSTEP
        mov.i   }tim2,          >tim2
cel2    spl     @cel2,          }CSTEP
        mov.i   }cel2,          >cel2
        mov.i   <-FSTEP,        {FSTEP
        mov.i   {cel2,          <ncl2
ncl2    jmp     @ncl2,          >NSTEP

end warrior

