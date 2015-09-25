/****************************************************************************
*
*   lib.rs
*   frio
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate libc;

pub mod net;
mod app;
mod types;
mod context;
mod win32;

use std::rc::Rc;

pub use app::App;
pub use context::Context;


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

pub trait TcpReceiveNotify {
    fn tcp_receive (
        &mut self,
        data: &[u8],
        conn: &mut net::TcpConnection
    );
}

pub trait TcpConnectNotify {
    fn tcp_connect (&mut self) -> TcpReceiveNotify;
}
