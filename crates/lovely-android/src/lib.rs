mod lualib;

use lovely_core::log::*;
use lovely_core::sys::LuaState;
use lualib::LUA_LIBRARY;
use std::{ffi::c_void, mem, panic, sync::{LazyLock, OnceLock}};

use jni::{JNIVersion, JavaVM};
use jni::sys::jint;

use lovely_core::Lovely;

static RUNTIME: OnceLock<&'static Lovely> = OnceLock::new();

static RECALL: LazyLock<
    unsafe extern "C" fn(*mut LuaState, *const u8, usize, *const u8, *const u8) -> u32,
> = LazyLock::new(|| unsafe {
    let lua_loadbufferx: unsafe extern "C" fn(
        *mut LuaState,
        *const u8,
        usize,
        *const u8,
        *const u8,
    ) -> u32 = *LUA_LIBRARY.get(b"luaL_loadbufferx").unwrap();
    let orig = dobby_rs::hook(
        lua_loadbufferx as *mut c_void,
        lua_loadbufferx_detour as *mut c_void,
    )
    .unwrap();
    mem::transmute(orig)
});

unsafe extern "C" fn lua_loadbufferx_detour(
    state: *mut LuaState,
    buf_ptr: *const u8,
    size: usize,
    name_ptr: *const u8,
    mode_ptr: *const u8,
) -> u32 {
    let rt = RUNTIME.get().unwrap_unchecked();
    rt.apply_buffer_patches(state, buf_ptr, size, name_ptr, mode_ptr)
}

#[no_mangle]
#[allow(non_snake_case)]
unsafe extern "C" fn luaL_loadbuffer(
    state: *mut LuaState,
    buf_ptr: *const u8,
    size: usize,
    name_ptr: *const u8,
) -> u32 {
    let rt = RUNTIME.get().unwrap_unchecked();
    rt.apply_buffer_patches(state, buf_ptr, size, name_ptr, std::ptr::null())
}

#[allow(non_snake_case)]
#[no_mangle]
unsafe extern "C" fn JNI_OnLoad(_jvm: JavaVM, _: *mut c_void) -> jint {
    panic::set_hook(Box::new(|x| {
        let message = format!("lovely-injector has crashed: \n{x}");
        error!("{message}");
    }));

    std::env::set_var("LOVELY_MOD_DIR", "/sdcard/Documents/LovelyMods");

    let rt = Lovely::init(&|a, b, c, d, e| RECALL(a, b, c, d, e), lualib::get_lualib(), false);
    RUNTIME
        .set(rt)
        .unwrap_or_else(|_| panic!("Failed to instantiate runtime."));

    let lua_loadbuffer: unsafe extern "C" fn(
        *mut LuaState,
        *const u8,
        isize,
        *const u8,
    ) -> u32 = *LUA_LIBRARY.get(b"luaL_loadbuffer").unwrap();
    let _ = dobby_rs::hook(
        lua_loadbuffer as *mut c_void,
        luaL_loadbuffer as *mut c_void,
    )
    .unwrap();

    JNIVersion::V4.into()
}
