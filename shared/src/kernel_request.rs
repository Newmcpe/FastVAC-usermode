use std::mem::size_of;
use std::sync::Mutex;
use std::thread::sleep;

use windows::core::imp::CloseHandle;
use windows::core::PCSTR;
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{CreateFileA, FILE_ATTRIBUTE_NORMAL, FILE_BEGIN, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadFile, SetFilePointer, WriteFile};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct INFORMATION {
    pub operation: i64,
    pub src_addr: u64,
    pub dst_addr: u64,
    pub size: usize,
    pub client_base: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CommunicationDTO {
    request: INFORMATION,
    mode: i32,
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::
        mem::size_of::<T>(),
    )
}

pub struct Driver {
    pub file_name: Mutex<String>,
}

impl Driver {
    pub fn init() -> Self {
        Self {
            file_name: Mutex::new("C:\\Users\\Newmcpe\\Desktop\\mrpenis.log.txt".to_string()),
        }
    }

    pub fn get_clientdll_base(&self) -> u64 {
        let file_name = self.file_name.lock().unwrap();

        let request = INFORMATION {
            operation: 0x1,
            src_addr: 0,
            dst_addr: 0,
            size: 256,
            client_base: 0,
        };

        let out_dto = CommunicationDTO {
            request,
            mode: 1,
        };
        let file = unsafe {
            CreateFileA(
                PCSTR::from_raw((file_name.to_string() + "\0").as_ptr()),
                (GENERIC_WRITE | GENERIC_READ).0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            ).expect("TODO: panic message")
        };

        unsafe { WriteFile(file, Some(any_as_u8_slice(&out_dto)), None, None).unwrap() };

        loop {
            unsafe { SetFilePointer(file, 0, None, FILE_BEGIN); }

            let mut buffer = [0u8; size_of::<CommunicationDTO>()];
            let mut bytesRead = 0;

            let read_result = unsafe { ReadFile(file, Some(&mut buffer), Some(&mut bytesRead), None) };
            if read_result.is_err() {
                println!("Error reading file");
                sleep(std::time::Duration::from_millis(100));
                continue;
            }

            let received_dto: CommunicationDTO = unsafe { std::mem::transmute(buffer) };
            if received_dto.mode != 2 {
                dbg!(received_dto);
                sleep(std::time::Duration::from_millis(1000));
                continue;
            }

            //    unsafe { fclose(file); };
            CloseHandle(file);
            return received_dto.request.client_base;
        }
    }
}