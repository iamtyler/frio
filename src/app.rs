/****************************************************************************
*
*   core.rs
*   frio
*
*   Copyright 2015 Tyler Cole
*
***/

use std::rc::Rc;
use std::ptr;
use std::result;

use win32;


/****************************************************************************
*
*   Types
*
***/

#[cfg(target_pointer_width = "32")]
pub type Handle = u32;

#[cfg(target_pointer_width = "64")]
pub type Handle = u64;

pub type Result<T> = result::Result<T, u32>;


/****************************************************************************
*
*   Client code
*
***/

pub trait Startup {
    fn start (
        &mut self,
        core: Rc<App>
    );
}

pub trait Cleanup {
    fn cleanup_start (&mut self);
    fn cleanup_is_complete (&mut self) -> bool { return true; }
}

pub trait Task {
    fn task_run (&mut self);
}


/****************************************************************************
*
*   EventNotify
*
***/

pub trait EventNotify {
    fn on_app_event (&mut self, app: Rc<App>);
}

pub struct EventData {
    pub overlapped: win32::OVERLAPPED,
    pub offset: usize,
}

impl EventData {
    pub fn new () -> EventData {
        EventData {
            overlapped: win32::OVERLAPPED::new(),
            offset: 0,
        }
    }
}


/****************************************************************************
*
*   App
*
***/

pub struct App {
    handle: Handle,
}

impl App {
    //=======================================================================
    pub fn run<S: Startup> (mut startup: S) {
        let handle;
        unsafe {
            handle = win32::CreateIoCompletionPort(
                win32::INVALID_HANDLE_VALUE,
                win32::NULL_HANDLE,
                0,
                0
            );
        }
        if handle.is_null() {
            return;
        }

        let core = Rc::new(App {
            handle: handle as Handle,
        });
        startup.start(core.clone());

        // TODO: calculate overlapped offset from notify struct

        let mut overlapped: *mut win32::OVERLAPPED = ptr::null_mut();
        let mut bytes: u32 = 0;
        let mut key: win32::ULONG_PTR = 0;
        loop {
            let success: bool;
            unsafe {
                success = win32::GetQueuedCompletionStatus(
                    core.handle as win32::HANDLE,
                    &mut bytes as *mut u32,
                    &mut key as *mut win32::ULONG_PTR,
                    &mut overlapped as *mut *mut win32::OVERLAPPED,
                    win32::INFINITE
                ) == 1;
            }

            // TODO: handle errors better
            assert!(success);

            // TODO: pull notify from overlapped
        }
    }

    //=======================================================================
    pub fn stop (self) {
        // TODO: figure out how to stop this thing
    }

    //=======================================================================
    pub fn cleanup_register (&mut self, cleanup: Rc<Cleanup>) {
        let _ = cleanup;
    }

    //=======================================================================
    pub fn task_queue (&mut self, task: Box<Task>) {
        let _ = task;
    }

    //=======================================================================
    pub fn link (&mut self, handle: Handle) -> bool {
        return unsafe {
            win32::CreateIoCompletionPort(
                handle as win32::HANDLE,
                self.handle as win32::HANDLE,
                0, // TODO: pass in link notify
                0
            ) as Handle
        } == self.handle;
    }
}


/****************************************************************************
*
*   Public functions
*
***/
