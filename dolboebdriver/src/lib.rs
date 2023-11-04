#![no_std]

extern crate alloc;

use core::ffi::{c_int};
use windows_kernel_rs::device::{
    Completion, Device, DeviceDoFlags, DeviceFlags, DeviceOperations, DeviceType, RequestError};
use windows_kernel_rs::{Access, Driver, Error, IoControlRequest, kernel_module, KernelModule, println, RequiredAccess, SymbolicLink};
use windows_kernel_rs::process::Process;

struct DolboebDevice {
    value: u32,
}

struct MemoryReadRequest {
    address: u64,
}

const IOCTL_PRINT_VALUE: u32 = 0x800;
const IOCTL_READ_VALUE: u32 = 0x801;
const IOCTL_WRITE_VALUE: u32 = 0x802;

impl DolboebDevice {
    fn print_value(&mut self, _request: &IoControlRequest) -> Result<u32, Error> {
        let process = Process::by_id(3048).unwrap();
        let base_address = process.get_base_address();

        println!("base_address: {:x?}", base_address);

        Ok(0)
    }

    fn read_value(&mut self, request: &IoControlRequest) -> Result<u32, Error> {
        let mut user_ptr = request.user_ptr();

        user_ptr.write(&self.value)?;
        Ok(core::mem::size_of::<u32>() as u32)
    }

    fn write_value(&mut self, request: &IoControlRequest) -> Result<u32, Error> {
        let user_ptr = request.user_ptr();

        self.value = user_ptr.read()?;

        Ok(0)
    }
}

impl DeviceOperations for DolboebDevice {
    fn ioctl(&mut self, _device: &Device, request: IoControlRequest) -> Result<Completion, RequestError> {
        let result = match request.function() {
            (_, IOCTL_PRINT_VALUE) =>
                self.print_value(&request),
            (RequiredAccess::READ_DATA, IOCTL_READ_VALUE) =>
                self.read_value(&request),
            (RequiredAccess::WRITE_DATA, IOCTL_WRITE_VALUE) =>
                self.write_value(&request),
            _ => Err(Error::INVALID_PARAMETER),
        };

        match result {
            Ok(size) => Ok(Completion::Complete(size, request.into())),
            Err(e) => Err(RequestError(e, request.into())),
        }
    }
}

struct Module {
    _device: Device,
    _symbolic_link: SymbolicLink,
}

impl KernelModule for Module {
    fn init(mut driver: Driver, _: &str) -> Result<Self, Error> {
        let device = driver.create_device(
            "\\Device\\Womic",
            DeviceType::Unknown,
            DeviceFlags::SECURE_OPEN,
            DeviceDoFlags::DO_BUFFERED_IO,
            Access::NonExclusive,
            DolboebDevice {
                value: 0,
            },
        )?;
        let symbolic_link = SymbolicLink::new("\\??\\Womic", "\\Device\\Womic")?;

        println!("Driver initialized");

        Ok(Module {
            _device: device,
            _symbolic_link: symbolic_link,
        })
    }

    fn cleanup(&mut self, _driver: Driver) {
        println!("Driver unloaded");
    }
}

extern "system" {
    fn KeBugCheckEx(
        BugCheckCode: c_int,
        BugCheckParameter1: Option<usize>,
        BugCheckParameter2: Option<usize>,
        BugCheckParameter3: Option<usize>,
        BugCheckParameter4: Option<usize>,
    );
}

kernel_module!(Module);
