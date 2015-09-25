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

use types::Handle;
use types::Endpoint;
use win32;


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
    pub fn run<S: super::Startup> (mut startup: S) {
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

        let mut overlapped: *mut win32::OVERLAPPED = ptr::null_mut();
        let mut bytes: u32 = 0;
        let mut key: win32::ULONG_PTR = 0;
        loop {
            let mut success: bool;
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
    pub fn cleanup_register (&mut self, cleanup: Rc<super::Cleanup>) {
        let _ = cleanup;
    }

    //=======================================================================
    pub fn task_queue (&mut self, task: Box<super::Task>) {
        let _ = task;
    }

    //=======================================================================
    pub fn tcp_listen<N: super::TcpConnectNotify> (
        &mut self,
        endpoint: Endpoint,
        notify: N
    ) {
        let _ = notify;
        let _ = endpoint;
    }

    //=======================================================================
    fn associate (&self, handle: Handle) {

    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn associate (app: Rc<App>, handle: Handle) {
    app.associate(handle);
}
