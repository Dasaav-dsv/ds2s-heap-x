use std::ops::Shr;

use windows::{
    core::{Error as WindowsError, Result as WindowsResult, PCWSTR},
    Win32::{
        Foundation::{GetLastError, ERROR_BAD_LENGTH},
        Storage::FileSystem::VS_FIXEDFILEINFO,
        System::LibraryLoader::{FindResourceW, LoadResource, LockResource},
    },
};

pub fn verify() -> bool {
    get_file_version().is_ok_and(|v| {
        v == Version {
            major: 1,
            minor: 0,
            revision: 3,
            build: 0,
        }
    })
}

#[derive(Debug, PartialEq)]
struct Version {
    major: u16,
    minor: u16,
    revision: u16,
    build: u16,
}

fn get_file_version() -> WindowsResult<Version> {
    // Resource: VS_VERSION, resource type: RT_VERSION.
    let resource_handle =
        unsafe { FindResourceW(None, PCWSTR::from_raw(1 as _), PCWSTR::from_raw(16 as _)) };

    if resource_handle.is_invalid() {
        return Err(WindowsError::new(
            unsafe { GetLastError() }.to_hresult(),
            "version resource not found",
        ));
    }

    let global_handle = unsafe { LoadResource(None, resource_handle)? };

    let mut resource_start = unsafe { LockResource(global_handle).cast::<u32>() };

    if resource_start.is_null() {
        return Err(WindowsError::new(
            unsafe { GetLastError() }.to_hresult(),
            "LockResource returned null",
        ));
    }

    let resource_end = unsafe { resource_start.byte_add(resource_start.read_unaligned() as usize) };
    resource_start = unsafe { resource_start.add(1) };

    const VERSION_SIGNATURE: u32 = 0xFEEF04BD;

    while resource_start.addr() < resource_end.addr() {
        if unsafe { resource_start.read_unaligned() } == VERSION_SIGNATURE {
            break;
        } else {
            resource_start = unsafe { resource_start.add(1) };
        }
    }

    if resource_start.addr() + std::mem::size_of::<VS_FIXEDFILEINFO>() > resource_end.addr() {
        return Err(WindowsError::new(
            ERROR_BAD_LENGTH.to_hresult(),
            "invalid version resource length",
        ));
    }

    let resource = resource_start.cast::<VS_FIXEDFILEINFO>();

    let version_ms = unsafe { (&raw const (*resource).dwFileVersionMS).read_unaligned() };
    let version_ls = unsafe { (&raw const (*resource).dwFileVersionLS).read_unaligned() };

    Ok(Version {
        major: version_ms.shr(16) as u16,
        minor: version_ms as u16,
        revision: version_ls.shr(16) as u16,
        build: version_ls as u16,
    })
}
