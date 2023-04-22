use crate::{cpu, process};
use core::convert::{TryFrom, TryInto};
use alloc::collections::vec_deque::VecDeque;
use crate::process::{State, TMR_VALUES_LIST};

#[repr(usize)]
pub enum Syscall {
    Nop = 0,
    DumpRegisters,
    Sleep,
    Exit,
    TmrAdd,
    Verify,
    PrintTotal
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
            _ => Err(()),
        }
    }
}

extern "C" {
    fn _make_syscall(
        sysno: usize, // n da chamada
        arg0: Option<VecDeque<i32>>,
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
        Ok(Syscall::TmrAdd) => {
            crate::println!(" aaaaaaaaa");
            if let Some(mut tmr) = TMR_VALUES_LIST.take(){
                crate::println!("Total value: {:?}", process::total);
                let mut vec_deque = VecDeque::new();
                vec_deque.push_back(1);
                vec_deque.push_back(1);
                vec_deque.push_back(1);
                tmr.push_back(Some(vec_deque));
                TMR_VALUES_LIST.replace(tmr);
                if let Some(new_tmr) = TMR_VALUES_LIST.as_ref() {
                    crate::println!("TMR_VALUES_LIST size: {}", new_tmr.len());
                    if(new_tmr.len() >= 3){
                        syscall_verify();
                    }
                }
            }
        }
        Ok(Syscall::Verify) => {
            if let Some(tmr) = TMR_VALUES_LIST.as_ref() {
                let mut max_count = 0;
                let mut most_common_value = 0;
            
                for i in 0..tmr.len() {
                    let mut count = 0;
                    for j in i+1..tmr.len() {
                        if tmr[i] == tmr[j] {
                            count += 1;
                        }
                    }
                    if count > max_count {
                        max_count = count;
                        most_common_value = 0;
                    }
                }
            
                crate::println!("Correct output: {}", most_common_value);
            }
        }
        Err(_) => panic!("Unknown syscall {}", syscall_id),
    }
}

pub fn syscall_nop() -> usize {
    unsafe { _make_syscall(Syscall::Nop as usize, None, 0, 0, 0, 0, 0) }
}

pub fn syscall_dump() -> usize {
    unsafe { _make_syscall(Syscall::DumpRegisters as usize, None, 0, 0, 0, 0, 0) }
}

pub fn syscall_sleep() -> usize {
    unsafe { _make_syscall(Syscall::Sleep as usize, None, 0, 0, 0, 0, 0) }
}

pub fn syscall_exit() -> usize {
    unsafe { _make_syscall(Syscall::Exit as usize, None, 0, 0, 0, 0, 0) }
}

pub fn syscall_push_tmr(total: Option<VecDeque<i32>> ) -> usize {
    unsafe { _make_syscall(Syscall::TmrAdd as usize, total, 0, 0, 0, 0, 0) }
}
pub fn syscall_print_total(total: Option<VecDeque<i32>> ) -> usize {
    unsafe { _make_syscall(Syscall::PrintTotal as usize, total, 0, 0, 0, 0, 0) }
}

pub fn syscall_verify() -> usize {
    unsafe { _make_syscall(Syscall::Verify as usize, None, 0, 0, 0, 0, 0) }
}