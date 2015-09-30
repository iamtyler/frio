/****************************************************************************
*
*   net/mod.rs
*   frio
*
*   Copyright 2015 Tyler Cole
*
***/

mod tcp;
mod endpoint;

use std::result;

pub use self::endpoint::*;
pub use self::tcp::TcpConnectNotify;
pub use self::tcp::TcpReceiveNotify;
pub use self::tcp::TcpConnection;
pub use self::tcp::TcpListener;

use win32;


/****************************************************************************
*
*   Types
*
***/

pub type Result<T> = result::Result<T, u32>;


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn initialize () -> Result<()> {
    let mut data = win32::WSAData {
        wVersion: 0,
        wHighVersion: 0,
        szDescription: [0; win32::WSADESCRIPTION_LEN + 1],
        szSystemStatus: [0; win32::WSASYS_STATUS_LEN + 1],
        iMaxSockets: 0,
        iMaxUdpDg: 0,
        lpVendorInfo: 0 as *mut u8,
    };

    if unsafe { win32::WSAStartup(
        2 + (2 << 8),
        &mut data as *mut win32::WSAData
    ) } != 0 {
        return tcp::get_error();
    }

    // TODO: verify version

    return Ok(());
}

//===========================================================================
pub fn cleanup () {
    // TODO: return detailed error
    unsafe {
        win32::WSACleanup();
    }
}
