#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum State {
    Ready,
    Running,
    Sleeping,
    Waiting,
    Dead,
}

use crate::{arch, consts, syscall};
use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use core::{fmt, ptr::null_mut};
use riscv::register::mcycle;

static mut NEXT_PID: usize = 1;

#[derive(Debug, Clone)]
pub struct ProcessData {
    cwd_path: [u8; 128],
}

impl ProcessData {
    pub fn zero() -> Self {
        ProcessData { cwd_path: [0; 128] }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Process {
    pub state: State,
    pub pid: usize,
    pub is_tmr: bool,
    stack: *mut u8,
    frame: *mut arch::isa::trap::TrapFrame,
    root: *mut arch::isa::page::Table,
    data: ProcessData,
    program: *mut u8,
    sleep_until: usize,
}

pub static mut PROCESS_LIST: Option<VecDeque<Process>> = None;
pub static mut TMR_VALUES_LIST: Option<VecDeque<Option<VecDeque<usize>>>> = None;
pub static mut time_total:f32 = 0.0;
pub static mut total: Option<VecDeque<usize>> = None;
pub static mut count:u32 = 0;
pub static mut TMR_BOOL:bool = false;

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Process of pid {}, frame {:p}", self.pid, self.frame)
    }
}

fn do_nothing() {}

impl Process {
    pub fn get_frame_addr(&self) -> usize {
        self.frame as usize
    }
    pub fn get_pc(&self) -> usize {
        unsafe { (*self.frame).pc }
    }
    pub fn get_table_addr(&self) -> usize {
        self.root as usize
    }
    pub fn get_state(&self) -> &State {
        &self.state
    }
    pub fn get_pid(&self) -> usize {
       self.pid
    }
    pub fn clones(&self) -> Self {
        let cloned = Process {
            is_tmr: false,
            frame: arch::mem::zalloc(1) as *mut arch::isa::trap::TrapFrame,
            stack: arch::mem::alloc(consts::STACK_PAGES),
            pid: unsafe { NEXT_PID },
            root: arch::mem::zalloc(1) as *mut arch::isa::page::Table,
            state: State::Running,
            data: ProcessData::zero(),
            program: null_mut(),
            sleep_until: 0,
        };
    
        unsafe {
            NEXT_PID += 1;
        }
    
    cloned
    }
    pub fn tmr(&self) -> Option<(Self, Self, Self)> {
        if self.is_tmr {
            let clone1 = self.clones();
            let clone2 = self.clones();
            let clone3 = self.clones();
            Some((clone1, clone2, clone3))
        } else {
            None
        }
    }
    pub fn sleep(&mut self, duration: usize) {
        self.state = State::Sleeping;
        self.sleep_until = duration;
    }
}


pub fn create_process(func: fn(), tmr: bool) -> usize {
    let func_addr = func as usize;
    let func_vaddr = func_addr; //- 0x6000_0000;

    let mut pid = 0;

    let ret_proc = Process {
        is_tmr: tmr,
        frame: arch::mem::zalloc(1) as *mut arch::isa::trap::TrapFrame,
        stack: arch::mem::alloc(consts::STACK_PAGES),
        pid: unsafe { NEXT_PID },
        root: arch::mem::zalloc(1) as *mut arch::isa::page::Table,
        state: State::Running,
        data: ProcessData::zero(),
        program: null_mut(),
        sleep_until: 0,
    };

    if(!ret_proc.is_tmr) {

        unsafe {
            NEXT_PID += 1;
        }

        arch::frame::create_process(
            unsafe { &mut *ret_proc.frame },
            func_vaddr,
            ret_proc.stack as usize,
            do_nothing as usize,
            ret_proc.pid as usize,
        );

        if let Some(mut pl) = unsafe { PROCESS_LIST.take() } {
            let pid = ret_proc.pid;
            pl.push_back(ret_proc);

            unsafe {
                PROCESS_LIST.replace(pl);
            }

        }

    } else {
    
    let mut processes: Option<VecDeque<Process>> = None;

        unsafe {
            TMR_BOOL = true;

            processes = Some(VecDeque::with_capacity(3));
        
            let tmr_values_list = ret_proc.tmr();
            for (p1, p2, p3) in tmr_values_list {
                processes.as_mut().unwrap().push_back(p1);
                processes.as_mut().unwrap().push_back(p2);
                processes.as_mut().unwrap().push_back(p3);
            }
            
        }
        
        for process in processes.as_mut().unwrap() {
            arch::frame::create_process(
                unsafe { &mut *process.frame },
                func_vaddr,
                process.stack as usize,
                do_nothing as usize,
                process.pid as usize,
            );

            if let Some(mut pl) = unsafe { PROCESS_LIST.take() } {
                pid = (*process).pid;
                pl.push_back(process.clone());

                unsafe {
                    PROCESS_LIST.replace(pl);
                }
                
            } else {
                return 0;
            }
            unsafe {
                NEXT_PID += 1;
            }
        
        }
        
    }   
    pid
}


pub fn sleep_pid(pid: usize, duration: usize) -> bool {
    unsafe {
        if let Some(mut pl) = PROCESS_LIST.take() {
            for proc in pl.iter_mut() {
                if proc.pid == pid {
                    proc.sleep(duration);
                    return true;
                }
            }
        }
    }
    false
}

impl Drop for Process {
    fn drop(&mut self) {
        arch::mem::dealloc(self.stack);

        unsafe {
            arch::mem::unmap(&mut *self.root);
        }

        arch::mem::dealloc(self.root as *mut u8);
    }
}

pub fn init_tmr_values_list() {
    unsafe {
        TMR_VALUES_LIST = Some(VecDeque::with_capacity(3));
    }
}



pub fn init() -> usize {
    unsafe {
        let start_time = mcycle::read();

        PROCESS_LIST = Some(VecDeque::with_capacity(15));

        init_tmr_values_list();

        create_process(sum, true);

        let pl = PROCESS_LIST.take().unwrap();
        let p = pl.front().unwrap().frame;

        PROCESS_LIST.replace(pl);

        let end_time = mcycle::read();
        let (execution_time, _) = end_time.overflowing_sub(start_time);

        const CLOCK_FREQUENCY: f32 = 100_000_000.0; // 100 MHz
        let execution_time_sec = (execution_time as f32) / CLOCK_FREQUENCY;

        time_total = execution_time_sec * 1000.0;

        (*p).pc
    }

}

fn sum() {
    syscall::syscall_sum();
}



fn init_process() {
    syscall::syscall_dump();
    syscall::syscall_exit();
}
  
