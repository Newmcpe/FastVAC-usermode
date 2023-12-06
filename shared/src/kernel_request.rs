use std::intrinsics::transmute;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::ptr::null;

use windows::core::s;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA};

pub type fNtUserGetPointerProprietaryId = unsafe extern "fastcall" fn(*const c_void) -> u64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct INFORMATION {
    pub operation: i64,
    pub src_addr: u64,
    pub dst_addr: u64,
    pub size: usize,
    pub client_base: u64,
}

pub struct Driver {
    pub NtUserGetPointerProprietaryId: fNtUserGetPointerProprietaryId,
}

impl Driver {
    pub fn init() -> Self {
        unsafe { LoadLibraryA(s!("win32u.dll")) }.unwrap();
        unsafe { LoadLibraryA(s!("user32.dll")) }.unwrap();

        let NtUserGetPointerProprietaryId = unsafe {
            let win32u = GetModuleHandleA(s!("win32u.dll")).unwrap();
            let NtUserGetPointerProprietaryId = GetProcAddress(win32u, s!("NtUserGetPointerProprietaryId"));
            transmute::<_, fNtUserGetPointerProprietaryId>(NtUserGetPointerProprietaryId.unwrap())
        };

        Self {
            NtUserGetPointerProprietaryId,
        }
    }

    pub fn readvm(&self, operation: i64, src_addr: u64, dst_addr: u64, size: usize) {
        let information = INFORMATION {
            operation,
            src_addr,
            dst_addr,
            size,
            client_base: unsafe { *null() },
        };
        let NtUserGetPointerProprietaryId = self.NtUserGetPointerProprietaryId;

        unsafe {
            NtUserGetPointerProprietaryId(&information as *const INFORMATION as *const c_void);
        }
    }

    pub fn readv<T>(&self, src: u64) -> T {
        let size = std::mem::size_of::<T>();
        let mut buffer: MaybeUninit<T> = MaybeUninit::<T>::uninit();
        self.readvm(0x80000001, src, &mut buffer as *mut _ as u64, size);
        unsafe { buffer.assume_init() }
    }

   // pub fn get_client_address
}