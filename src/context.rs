/****************************************************************************
*
*   context.rs
*   frio
*
*   Copyright 2015 Tyler Cole
*
***/

use app;
use net;


/****************************************************************************
*
*   Context
*
***/

pub struct Context;

impl Context {
    //=======================================================================
    pub fn new () -> Result<Context, u32> {
        if let Err(code) = net::initialize() {
            return Err(code);
        }

        return Ok(Context);
    }

    //=======================================================================
    pub fn run_app<S: app::Startup> (&mut self, startup: S) {
        app::App::run(startup);
    }
}

impl Drop for Context {
    //=======================================================================
    fn drop (&mut self) {
        net::cleanup();
    }
}
