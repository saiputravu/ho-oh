use std::{
    fs::File,
    io::{self, Read},
    sync::RwLockReadGuard,
};

use dispatch2::DispatchData;
use objc2_foundation::NSString;
use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

pub fn setup_device()
-> Result<objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn MTLDevice>>, String> {
    MTLCreateSystemDefaultDevice().ok_or("unable to create device".to_string())
}

#[derive(Debug)]
enum LoadKernelFileError {
    IoError(std::io::Error),
}

pub fn load_kernel_file(
    device: objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn MTLDevice>>,
    filename: String,
    libname: String,
) -> Result<(), LoadKernelFileError> {
    let file = File::open(filename)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer);

    let data = DispatchData::from_bytes(buffer.as_bytes());
    let library = device.newLibraryWithData_error(&data)?;
    let function_name = NSString::from_str(libname.as_str());
    Ok(())
}
