#[allow(dead_code)]
#[derive(Debug)]
pub enum State {
    Ready,
    Running,
    Sleeping,
    Waiting,
    Dead,
}

use crate::{arch, consts, syscall};
use alloc::collections::vec_deque::VecDeque;
use core::{fmt, ptr::null_mut};

static mut NEXT_PID: usize = 1;

#[derive(Debug)]
pub struct ProcessData {
    cwd_path: [u8; 128],
}

impl ProcessData {
    pub fn zero() -> Self {
        ProcessData { cwd_path: [0; 128] }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Process {
    pub state: State,
    pub pid: usize,

    stack: *mut u8,
    frame: *mut arch::isa::trap::TrapFrame,

    root: *mut arch::isa::page::Table,
    data: ProcessData,
    program: *mut u8,
    sleep_until: usize,
}

pub static mut PROCESS_LIST: Option<VecDeque<Process>> = None;

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

    pub fn sleep(&mut self, duration: usize) {
        self.state = State::Sleeping;
        self.sleep_until = duration;
    }
}

pub fn create_process(func: fn()) -> usize {
    let func_addr = func as usize;
    let func_vaddr = func_addr; //- 0x6000_0000;

    let ret_proc = Process {
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

        pid
    } else {
        0
    }
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

pub fn init() -> usize {
    unsafe {
        PROCESS_LIST = Some(VecDeque::with_capacity(15));

        create_process(init_process);
        create_process(init_proces2);

        let pl = PROCESS_LIST.take().unwrap();
        let p = pl.front().unwrap().frame;

        PROCESS_LIST.replace(pl);
        (*p).pc
    }
}

fn init_process() {
    syscall::syscall_dump();
    syscall::syscall_exit();
}

fn init_proces2() {
    loop {}
}
