#![no_std]

#![feature(alloc_error_handler)]

extern crate alloc;

pub mod allocator;
pub mod device;
pub mod driver;
pub mod error;
pub mod io;
pub mod string;

pub use crate::device::{Device, DeviceOperations, dispatch_device, UserPtr};
pub use crate::driver::Driver;
pub use crate::error::Error;

pub use widestring::U16CString;
pub use windows_kernel_sys::base::{DRIVER_OBJECT, IRP_MJ_MAXIMUM_FUNCTION, NTSTATUS, STATUS_SUCCESS, UNICODE_STRING};

#[global_allocator]
static ALLOCATOR: allocator::KernelAllocator = allocator::KernelAllocator;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[used]
#[no_mangle]
pub static _fltused: i32 = 0;

#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 {
    unimplemented!()
}

#[macro_export]
macro_rules! kernel_module {
    ($module:ty) => {
        static mut __MOD: Option<$module> = None;

        #[no_mangle]
        pub extern "system" fn driver_entry(
            driver: &mut $crate::DRIVER_OBJECT,
            registry_path: &$crate::UNICODE_STRING,
        ) -> $crate::NTSTATUS {
            unsafe {
                driver.DriverUnload = Some(driver_exit);

                for i in 0..$crate::IRP_MJ_MAXIMUM_FUNCTION {
                    driver.MajorFunction[i as usize] = Some($crate::dispatch_device);
                }
            }

            let driver = unsafe {
                Driver::from_raw(driver)
            };

            let registry_path = unsafe {
                $crate::U16CString::from_ptr_str(registry_path.Buffer)
            };
            let registry_path = registry_path.to_string_lossy();

            match <$module as $crate::KernelModule>::init(driver, registry_path.as_str()) {
                Ok(m) => {
                    unsafe {
                        __MOD = Some(m);
                    }

                    $crate::STATUS_SUCCESS
                }
                Err(e) => {
                    e.to_kernel_errno()
                }
            }
        }

        pub unsafe extern "C" fn driver_exit(
            driver: *mut $crate::DRIVER_OBJECT,
        ) {
            let driver = unsafe {
                Driver::from_raw(driver)
            };

            match __MOD.take() {
                Some(mut m) => m.cleanup(driver),
                _ => (),
            }
        }
    };
}

pub trait KernelModule: Sized + Sync {
    fn init(driver: Driver, registry_path: &str) -> Result<Self, Error>;
    fn cleanup(&mut self, driver: Driver);
}