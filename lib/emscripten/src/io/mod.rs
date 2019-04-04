#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use self::unix::*;

#[cfg(windows)]
pub use self::windows::*;

use wasmer_runtime_core::vm::Ctx;

/// getprotobyname
pub fn getprotobyname(_ctx: &mut Ctx, _name_ptr: i32) -> i32 {
    debug!("emscripten::getprotobyname");
    unimplemented!()
}

/// getprotobynumber
pub fn getprotobynumber(_ctx: &mut Ctx, _one: i32) -> i32 {
    debug!("emscripten::getprotobynumber");
    unimplemented!()
}

/// sigdelset
pub fn sigdelset(ctx: &mut Ctx, set: i32, signum: i32) -> i32 {
    debug!("emscripten::sigdelset {} {}", set, signum);
    // HEAP32[((set)>>2)]=HEAP32[((set)>>2)]& (~(1 << (signum-1)));
    let val = emscripten_memory_pointer!(ctx.memory(0), set) as *mut u32;
    unsafe {
        *val = *val & !(1 << (signum - 1)); // val & ~(1 << (signum-1))
    }
    0
}

/// sigfillset
pub fn sigfillset(ctx: &mut Ctx, set: i32) -> i32 {
    debug!("emscripten::sigfillset");
    // HEAP32[((set)>>2)]=-1>>>0;
    let val = emscripten_memory_pointer!(ctx.memory(0), set) as *mut u32;
    unsafe {
        *val = 4294967295; // -1>>>0
    }
    // unimplemented!()
    0
}

/// tzset
pub fn tzset(_ctx: &mut Ctx) {
    debug!("emscripten::tzset");
    unimplemented!()
}

/// strptime
pub fn strptime(_ctx: &mut Ctx, _one: i32, _two: i32, _three: i32) -> i32 {
    debug!("emscripten::strptime");
    unimplemented!()
}
