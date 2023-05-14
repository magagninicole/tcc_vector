use crate::{cpu, process};
use core::convert::{TryFrom, TryInto};
use alloc::collections::vec_deque::VecDeque;
use crate::process::{State, TMR_VALUES_LIST, total};

#[repr(usize)]
pub enum Syscall {
    Nop = 0,
    DumpRegisters,
    Sleep,
    Exit,
    TmrAdd,
    Verify,
    PrintTotal,
    Print,
    Sum
}

impl TryFrom<usize> for Syscall {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            x if x == Syscall::Nop as usize => Ok(Syscall::Nop),
            x if x == Syscall::DumpRegisters as usize => Ok(Syscall::DumpRegisters),
            x if x == Syscall::Sleep as usize => Ok(Syscall::Sleep),
            x if x == Syscall::Exit as usize => Ok(Syscall::Exit),
            x if x == Syscall::TmrAdd as usize => Ok(Syscall::TmrAdd),
            x if x == Syscall::Verify as usize => Ok(Syscall::Verify),
            x if x == Syscall::PrintTotal as usize => Ok(Syscall::PrintTotal),
            x if x == Syscall::Print as usize => Ok(Syscall::Print),
            x if x == Syscall::Sum as usize => Ok(Syscall::Sum),
            _ => Err(()),
        }
    }
}

extern "C" {
    fn _make_syscall(
        sysno: usize, // n da chamada
        arg0: usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
        arg5: usize,
    ) -> usize;
}

pub unsafe fn make_syscall(pc: usize, frame_ptr: *mut crate::arch::isa::trap::TrapFrame) {
    if frame_ptr.is_null() {
        return;
    }

    let frame = frame_ptr.as_mut().unwrap();
    let syscall_id = frame.syscall_id(); // processo

    // skip ecall
    frame.pc = pc + 4;
    match syscall_id.try_into() {
        Ok(Syscall::Nop) => {
            crate::println!("NOP");
        }
        Ok(Syscall::DumpRegisters) => {
            crate::println!("Registers");
            cpu::dump_registers(frame);
        }
        Ok(Syscall::Sleep) => {
            crate::println!("Sleeping");
            let proc = process::sleep_pid((*frame).pid, 1);
            crate::println!("Process is {}", proc);
        }
        Ok(Syscall::Exit) => {
            crate::println!("Exiting. Bye.");
            crate::abort()
        }
        Ok(Syscall::PrintTotal) => {
            crate::println!("Total: {:?}", process::total);
        }
        Ok(Syscall::Print) => {
            crate::println!("Execution time: {:?}", process::time_total);
        }
        Ok(Syscall::Sum) => {
            let mut x = VecDeque::new();
            let mut y = VecDeque::new();

            total = Some(VecDeque::with_capacity(3));

            x.push_back(1);
            x.push_back(2);
            x.push_back(3);
        
            y.push_back(1);
            y.push_back(2);
            y.push_back(3);
        
            if let Some(mut value) = total.take() {
                for i in 0..x.len() {
                    value.push_back(x[i] + y[i]);
                }
                total.replace(value);
            }
            crate::println!("Total value: {:?}", total);
            if(process::TMR_BOOL) {
             syscall_push_tmr();
             } else {
             syscall_print_total(); 
         }
              
        }
        
        Ok(Syscall::TmrAdd) => {
            if let Some(mut tmr) = TMR_VALUES_LIST.take(){
                tmr.push_back(total.clone());
                TMR_VALUES_LIST.replace(tmr);
                if let Some(new_tmr) = TMR_VALUES_LIST.as_ref() {
                    crate::println!("TMR_VALUES_LIST size: {} \n", new_tmr.len());
                    if(new_tmr.len() >= 3){
                        syscall_verify();
                    }
                }
            }
        }
        Ok(Syscall::Verify) => {
            if let Some(tmr) = TMR_VALUES_LIST.as_ref() {
                let mut max_count = 0;
                let mut most_common_value  = None;

                let mut x = 0;
                let mut y = 0;
        
                for i in 0..tmr.len() {
                    let mut count = 0;
                    for j in i+1..tmr.len() {
                        let mut count_queue = 0;
                        if let Some(queue) = tmr[i].as_ref() {
                            if let Some(line) = tmr[j].as_ref() {
                                for k in 0..queue.len(){
                                     x = queue[k];
                                     y = line[k];
                                    if(x == y) {
                                         crate::println!("x is {} ", x);
                                         crate::println!("y is {} ", y);
                                        count_queue +=1;
                                        crate::println!("count_queue is {} ", count_queue);
                                    } else {
                                        crate::println!("x is {} ", x);
                                        crate::println!("y is {} ", y);
                                    }
                                }
                            }
                            if(count_queue == queue.len()){
                                crate::println!("equal vector");
                                count += 1;
                            } else {
                                crate::println!("error");
                            }
                        }
                    }
                    if count > max_count {
                        max_count = count;
                        most_common_value = tmr[i].clone();
                    }
                }
                crate::println!("Most common value: {:?}", most_common_value);
                crate::println!("Execution time: {:?}", process::time_total);
            }
        
               
        }
        Err(_) => panic!("Unknown syscall {}", syscall_id),
    }
}

pub fn syscall_nop() -> usize {
    unsafe { _make_syscall(Syscall::Nop as usize, 0, 0, 0, 0, 0, 0) }
}

pub fn syscall_dump() -> usize {
    unsafe { _make_syscall(Syscall::DumpRegisters as usize, 0, 0, 0, 0, 0, 0) }
}

pub fn syscall_sleep() -> usize {
    unsafe { _make_syscall(Syscall::Sleep as usize, 0, 0, 0, 0, 0, 0) }
}

pub fn syscall_exit() -> usize {
    unsafe { _make_syscall(Syscall::Exit as usize, 0, 0, 0, 0, 0, 0) }
}

pub fn syscall_push_tmr() -> usize {
    unsafe { _make_syscall(Syscall::TmrAdd as usize, 0, 0, 0, 0, 0, 0) }
}
pub fn syscall_print_total() -> usize {
    unsafe { _make_syscall(Syscall::PrintTotal as usize, 0, 0, 0, 0, 0, 0) }
}
pub fn syscall_print() -> usize {
    unsafe { _make_syscall(Syscall::Print as usize, 0, 0, 0, 0, 0, 0) }
}

pub fn syscall_verify() -> usize {
    unsafe { _make_syscall(Syscall::Verify as usize, 0, 0, 0, 0, 0, 0) }
}

pub fn syscall_sum() -> usize {
    unsafe { _make_syscall(Syscall::Sum as usize, 0, 0, 0, 0, 0, 0) }
}
