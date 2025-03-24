use std::{
    ffi::{c_void, OsString},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
};

use windows::{
    core::{Error as WindowsError, IUnknown, Result as WindowsResult, GUID, HRESULT, HSTRING},
    Win32::{
        Foundation::{
            GetLastError, ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS,
            HINSTANCE, MAX_PATH,
        },
        System::{
            LibraryLoader::{GetModuleFileNameW, GetProcAddress, LoadLibraryW},
            SystemServices::DLL_PROCESS_ATTACH,
        },
    },
};

use crate::init_dll;

/// The DLL entry point.
///
/// For simplicity's sake (and consistency) does not follow the DllMain best practices.
///
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DllMain(hinst: HINSTANCE, reason: u32) -> i32 {
    dllmain_proxy(hinst, reason) as i32
}

/// dinput8.dll proxy export.
///
/// It's a legacy modengine requirement for its DLL chaining option (modengine will look for this export),
/// even if the DLL isn't named dinput8.dll or used for proxying.
///  
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn DirectInput8Create(
    hinst: HINSTANCE,
    version: u32,
    riid: *const GUID,
    out: *mut *mut c_void,
    unkouter: *mut IUnknown,
) -> HRESULT {
    direct_input8_create_proxy(hinst, version, riid, out, unkouter).into()
}

fn dllmain_proxy(hinst: HINSTANCE, reason: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        match get_dll_path(hinst) {
            Some(path) => init_dll(&PathBuf::from(path)),
            None => false,
        }
    } else {
        true
    }
}

fn get_dll_path(hinst: HINSTANCE) -> Option<OsString> {
    let mut size = MAX_PATH;
    let mut out = vec![0; size as usize];

    while out.len() < u32::MAX as usize {
        size = unsafe { GetModuleFileNameW(Some(hinst.into()), out.as_mut_slice()) };

        match unsafe { GetLastError() } {
            ERROR_INSUFFICIENT_BUFFER => {
                out.resize(out.len() * 2, 0);

                continue;
            }
            ERROR_SUCCESS => {
                out.resize(size as usize, 0);

                break;
            }
            _ => return None,
        }
    }

    Some(OsString::from_wide(&out))
}

fn direct_input8_create_proxy(
    hinst: HINSTANCE,
    version: u32,
    riid: *const GUID,
    out: *mut *mut c_void,
    unkouter: *mut IUnknown,
) -> WindowsResult<()> {
    let dinput8_path = get_dinput8_path().ok_or(WindowsError::new(
        ERROR_FILE_NOT_FOUND.to_hresult(),
        "failed to get dinput8.dll path",
    ))?;

    let dinput8 = unsafe { LoadLibraryW(&HSTRING::from(dinput8_path))? };

    let direct_input8_create = unsafe {
        std::mem::transmute::<
            _,
            fn(HINSTANCE, u32, *const GUID, *mut *mut c_void, *mut IUnknown) -> HRESULT,
        >(
            GetProcAddress(dinput8, windows::core::s!("DirectInput8Create")).ok_or_else(|| {
                WindowsError::new(
                    GetLastError().to_hresult(),
                    "GetProcAddress did not return a valid DirectInput8Create address",
                )
            })?,
        )
    };

    direct_input8_create(hinst, version, riid, out, unkouter).ok()
}

fn get_dinput8_path() -> Option<OsString> {
    std::env::var_os("SystemRoot").map(|mut path| {
        path.push("/system32/dinput8.dll");
        path
    })
}
