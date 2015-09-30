/****************************************************************************
*
*   net/tcp.rs
*   frio
*
*   Copyright 2015 Tyler Cole
*
***/

use libc;

use super::endpoint::*;
use super::super::app;
use super::super::win32;

use super::Result;

use std::mem;
use std::ptr;
use std::rc::Rc;


/****************************************************************************
*
*   Constants
*
***/

const ADDR_BYTES: usize = 16; // size of win32::sockaddr_in
const ADDR_BUFFER_BYTES: usize = ADDR_BYTES + 16;
const ADDRS_BUFFER_BYTES: usize = ADDR_BUFFER_BYTES * 2;


/****************************************************************************
*
*   Client code
*
***/

pub trait TcpReceiveNotify {
    fn on_tcp_receive (
        &mut self,
        data: &[u8],
        conn: &mut TcpConnection
    );
}

pub trait TcpConnectNotify {
    fn on_tcp_connect (&mut self) -> TcpReceiveNotify;
}


/****************************************************************************
*
*   Socket
*
***/

struct Socket {
    handle: app::Handle,
}

impl Socket {
    pub fn is_valid (&self) -> bool {
        self.handle != win32::INVALID_SOCKET
    }

    //=======================================================================
    pub fn new_invalid () -> Socket {
        Socket {
            handle: win32::INVALID_SOCKET,
        }
    }

    //=======================================================================
    pub fn new () -> Result<Socket> {
        let handle = unsafe { win32::socket(
                win32::AF_INET,
                win32::SOCK_STREAM,
                win32::IPPROTO_TCP
        ) };

        if handle == win32::INVALID_SOCKET {
            return get_error();
        }

        return Ok(Socket {
            handle: handle
        });
    }

    //=======================================================================
    pub fn bind (&mut self, endpoint: Endpoint) -> Result<()> {
        // TODO: support IPv6

        // Create sockaddr for binding
        let Endpoint::V4(ref v4) = endpoint;
        let octets = v4.address().octets();
        let port = ((v4.port() & 0xff) << 8) + ((v4.port() & 0xff00) >> 8);
        let sockaddr = win32::sockaddr_in {
            sin_family: win32::AF_INET as i16,
            sin_port: port,
            sin_addr: win32::in_addr {
                s_b1: octets[0],
                s_b2: octets[1],
                s_b3: octets[2],
                s_b4: octets[3],
            },
            sa_zero: [0; 8]
        };

        // Bind socket to address
        if unsafe { win32::bind(
            self.handle,
            &sockaddr as *const win32::sockaddr_in,
            mem::size_of::<win32::sockaddr_in>() as i32
        ) } != 0 {
            return get_error();
        }

        return Ok(());
    }

    //=======================================================================
    pub fn listen (&mut self) -> Result<()> {
        if unsafe { win32::listen(self.handle, win32::SOMAXCONN) } != 0 {
            return get_error();
        }
        return Ok(());
    }

    //=======================================================================
    pub fn close (&mut self) {
        if self.handle != win32::INVALID_SOCKET {
            unsafe { win32::closesocket(self.handle) };
            self.handle = win32::INVALID_SOCKET;
        }
    }
}

impl Drop for Socket {
    //=======================================================================
    fn drop (&mut self) {
        self.close();
    }
}


/****************************************************************************
*
*   TcpConnection
*
***/

pub struct TcpConnection {
    handle: app::Handle,
    local: Endpoint,
    remote: Endpoint,
    notify: Box<TcpReceiveNotify>,
}

impl TcpConnection {
    pub fn handle (&self) -> app::Handle { self.handle }
    pub fn endpoint_local (&self) -> &Endpoint { &self.local }
    pub fn endpoint_remote (&self) -> &Endpoint { &self.remote }

    //=======================================================================
    pub fn new (
        handle: app::Handle,
        local: Endpoint,
        remote: Endpoint,
        notify: Box<TcpReceiveNotify>
    ) -> TcpConnection {
        TcpConnection {
            handle: handle,
            local: local,
            remote: remote,
            notify: notify,
        }
    }

    //=======================================================================
    pub fn send (&mut self, data: &[u8]) {
        let _ = data;
        let _ = self.notify;
    }

    //=======================================================================
    pub fn close (self) {
    }
}


/****************************************************************************
*
*   DummyNotify
*
***/

// struct DummyNotify;

// impl TcpReceiveNotify for DummyNotify {
//     fn on_tcp_receive (
//         &mut self,
//         data: &[u8],
//         conn: &mut TcpConnection
//     ) {
//         let _ = data;
//         let _ = conn;
//     }
// }


/****************************************************************************
*
*   TcpListener
*
***/

pub struct TcpListener {
    socket: Socket,
    endpoint: Endpoint,
    notify: Box<TcpConnectNotify>,

    // Async accept data
    accept: Socket,
    addrs: [u8; ADDRS_BUFFER_BYTES],
    event: app::EventData,
}

impl TcpListener {
    pub fn endpoint (&self) -> &Endpoint { &self.endpoint }

    //=======================================================================
    pub fn new (
        app: &mut app::App,
        endpoint: Endpoint,
        notify: Box<TcpConnectNotify>
    ) -> Result<TcpListener> {
        // Create socket
        let socket = Socket::new();
        if let Err(code) = socket {
            return Err(code);
        }

        // Bind and listen
        let mut socket = socket.unwrap();
        if let Err(code) = socket.bind(endpoint) {
            return Err(code);
        }
        if let Err(code) = socket.listen() {
            return Err(code);
        }

        // Link handle to app
        if !app.link(socket.handle) {
            return Err(0);
        }

        // Create listener
        let mut listener = TcpListener {
            socket: socket,
            endpoint: endpoint,
            notify: notify,

            accept: Socket::new_invalid(),
            addrs: [0; ADDRS_BUFFER_BYTES],
            event: app::EventData::new(),
        };

        listener.event.offset = unsafe { (&mut(*(0 as *mut TcpListener)).event as *mut app::EventData) as usize };

        let _ = listener.notify;

        // Asynchronously accept connections
        if let Err(code) = listener.accept() {
            listener.close();
            return Err(code);
        }

        return Ok(listener);
    }

    //=======================================================================
    pub fn close (&mut self) {
        self.socket.close();
    }

    //=======================================================================
    fn accept (&mut self) -> Result<()> {
        // Proceed only if previous socket was accepted
        if self.accept.is_valid() {
            return Ok(());
        }

        // Get new socket
        let socket = Socket::new();
        if let Err(code) = socket {
            return Err(code);
        }
        self.accept = socket.unwrap();

        // Reset accept params
        self.event.overlapped = win32::OVERLAPPED::new();
        for b in self.addrs.iter_mut() {
            *b = 0;
        }

        // Call accept API
        unsafe {
            win32::AcceptEx(
                self.socket.handle,
                self.accept.handle,
                self.addrs[..ADDRS_BUFFER_BYTES].as_mut_ptr() as *mut libc::c_void,
                0,
                ADDR_BUFFER_BYTES as u32,
                ADDR_BUFFER_BYTES as u32,
                ptr::null_mut(),
                &mut self.event.overlapped as *mut win32::OVERLAPPED
            );
        }
        let code = get_error_code();
        if code == win32::ERROR_IO_PENDING as u32 {
            return Ok(());
        }
        else {
            return Err(0);
        }
    }
}

impl app::EventNotify for TcpListener {
    fn on_app_event (&mut self, app: Rc<app::App>) {
        let _ = app;

        // let local: *mut win32::sockaddr_in = self.addrs[0..16].as_mut_ptr() as *mut win32::sockaddr_in;
        // let remote: *mut win32::sockaddr_in = self.addrs[32..48].as_mut_ptr() as *mut win32::sockaddr_in;

        println!("TcpListener on_app_event");
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn get_error_code () -> u32 {
    return unsafe { win32::WSAGetLastError() };
}

//===========================================================================
pub fn get_error<T> () -> Result<T> {
    Err(get_error_code())
}
