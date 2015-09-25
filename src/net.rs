/****************************************************************************
*
*   net.rs
*   frio
*
*   Copyright 2015 Tyler Cole
*
***/

use std::mem;

use win32;

use super::types::Handle;
pub use super::types::IpAddrV4;
pub use super::types::EndpointV4;
pub use super::types::Endpoint;


/****************************************************************************
*
*   TcpConnection
*
***/

pub struct TcpConnection {
    handle: Handle,
    local: Endpoint,
    remote: Endpoint,
}

impl TcpConnection {
    pub fn handle (&self) -> Handle { self.handle }
    pub fn endpoint_local (&self) -> &Endpoint { &self.local }
    pub fn endpoint_remote (&self) -> &Endpoint { &self.remote }

    //=======================================================================
    pub fn new (
        handle: Handle,
        local: Endpoint,
        remote: Endpoint
    ) -> TcpConnection {
        TcpConnection {
            handle: handle,
            local: local,
            remote: remote,
        }
    }

    //=======================================================================
    pub fn send (&mut self, data: &[u8]) {
        let _ = data;
    }

    //=======================================================================
    pub fn close (self) {
    }
}


/****************************************************************************
*
*   TcpListener
*
***/

pub struct TcpListener {
    handle: Handle,
    endpoint: Endpoint,
}

impl TcpListener {
    pub fn endpoint (&self) -> &Endpoint { &self.endpoint }

    //=======================================================================
    pub fn new (endpoint: Endpoint) -> Option<TcpListener> {
        // Create socket
        let socket;
        unsafe {
            socket = win32::socket(
                win32::AF_INET,
                win32::SOCK_STREAM,
                win32::IPPROTO_TCP
            );
        }
        if socket == win32::INVALID_SOCKET {
            return None;
        }

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
        let mut result;
        unsafe {
            result = win32::bind(
                socket,
                &sockaddr as *const win32::sockaddr_in,
                mem::size_of::<win32::sockaddr_in>() as i32
            );
        }
        if result != 0 {
            // Close socket and return
            unsafe {
                win32::closesocket(socket);
            }
            return None;
        }

        // Listen
        unsafe {
            result = win32::listen(
                socket,
                win32::SOMAXCONN
            );
        }
        if result != 0 {
            // Close socket and return
            unsafe {
                win32::closesocket(socket);
            }
            return None;
        }

        // Return listener
        Some(TcpListener {
            handle: 0,
            endpoint: endpoint,
        })
    }

    //=======================================================================
    pub fn connect (&mut self) /* -> TcpConnection */ {
    }

    //=======================================================================
    pub fn close (self) {
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn initialize () -> bool {
    let mut data = win32::WSAData {
        wVersion: 0,
        wHighVersion: 0,
        szDescription: [0; win32::WSADESCRIPTION_LEN + 1],
        szSystemStatus: [0; win32::WSASYS_STATUS_LEN + 1],
        iMaxSockets: 0,
        iMaxUdpDg: 0,
        lpVendorInfo: 0 as *mut u8
    };

    let result;
    unsafe {
        result = win32::WSAStartup(
            2 + (2 << 8),
            &mut data as *mut win32::WSAData
        );
    }
    return result == 0;
}

//===========================================================================
pub fn cleanup () {
    unsafe {
        win32::WSACleanup();
    }
}
