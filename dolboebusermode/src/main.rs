use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::windows::io::AsRawHandle;
use winioctl::{DeviceType, Error};
use winioctl::{ioctl_none, ioctl_read, ioctl_write};

const IOCTL_PRINT_VALUE: u32 = 0x800;
const IOCTL_READ_VALUE:  u32 = 0x801;
const IOCTL_WRITE_VALUE: u32 = 0x802;

ioctl_none!(ioctl_print_value, DeviceType::Unknown, IOCTL_PRINT_VALUE);
ioctl_read!(ioctl_read_value, DeviceType::Unknown, IOCTL_READ_VALUE, i32);
ioctl_write!(ioctl_write_value, DeviceType::Unknown, IOCTL_WRITE_VALUE, i32);

fn main() -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open("\\??\\Womic")?;

    unsafe {
        ioctl_write_value(file.as_raw_handle(), &5)?;
    }

    let mut value = 0;

    unsafe {
        ioctl_read_value(file.as_raw_handle(), &mut value)?;
    }

    unsafe {
        ioctl_print_value(file.as_raw_handle()).expect("аааа");
    }

    print!("value: {}\n", value);

    Ok(())
}
