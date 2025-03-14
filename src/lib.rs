use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

mod winapi {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        // GetLogicalDriveStringsW
        pub fn GetLogicalDriveStringsW(
            n_buffer_length: u32,
            lp_buffer: *mut u16,
        ) -> u32;

        // QueryDosDeviceW
        pub fn QueryDosDeviceW(
            lp_device_name: *const u16,
            lp_target_path: *mut u16,
            ucch_max: u32,
        ) -> u32;

        // GetVolumePathNameW
        pub fn GetVolumePathNameW(
            lpsz_path_name: *const u16,
            lpsz_volume_path_name: *mut u16,
            cch_buffer_length: u32,
        ) -> i32;

        // GetFullPathNameW
        pub fn GetFullPathNameW(
            lp_file_name: *const u16,
            n_buffer_length: u32,
            lp_buffer: *mut u16,
            lp_file_part: *mut *mut u16,
        ) -> u32;
    }
}

lazy_static::lazy_static! {
    static ref MOUNT_POINTS: HashMap<String, String> = {
        get_mount_points().unwrap_or_else(|e| {
            eprintln!("Failed to initialize mount points: {}", e);
            HashMap::new()
        })
    };
}

type WinResult<T> = Result<T, io::Error>;

fn wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn from_wide_ptr(ptr: *const u16) -> WinResult<String> {
    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf16(slice)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-16 sequence"))
}

fn get_mount_points() -> WinResult<HashMap<String, String>> {
    let mut mount_points = HashMap::new();

    // 获取逻辑驱动器字符串长度
    let required_size = unsafe { winapi::GetLogicalDriveStringsW(0, ptr::null_mut()) };
    if required_size == 0 {
        return Err(io::Error::last_os_error());
    }

    // 分配缓冲区
    let mut buffer = vec![0u16; required_size as usize];
    let result = unsafe {
        winapi::GetLogicalDriveStringsW(
            buffer.len() as u32,
            buffer.as_mut_ptr()
        )
    };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    // 解析驱动器列表
    let mut current = 0;
    while current < buffer.len() {
        let start = current;
        while buffer[current] != 0 {
            current += 1;
        }
        if start == current {
            break;
        }

        let drive_wide = &buffer[start..current];
        let drive_str = String::from_utf16(drive_wide)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid drive string"))?;

        // 查询设备路径
        let device_name = wide_string(&drive_str[..2]); // 提取类似 "C:"
        let mut device_path = [0u16; 2048];
        let result = unsafe {
            winapi::QueryDosDeviceW(
                device_name.as_ptr(),
                device_path.as_mut_ptr(),
                device_path.len() as u32
            )
        };

        if result == 0 {
            let err = io::Error::last_os_error();
            eprintln!("Failed to query device for {}: {}", drive_str, err);
            continue;
        }

        let path = from_wide_ptr(device_path.as_ptr())?;
        mount_points.insert(drive_str, path);
        current += 1; // 跳过null终止符
    }

    Ok(mount_points)
}

fn get_volume_mount_point(path: &str) -> WinResult<String> {
    let path_wide = wide_string(path);

    // 获取绝对路径
    let mut full_path = [0u16; 4096];
    let len = unsafe {
        winapi::GetFullPathNameW(
            path_wide.as_ptr(),
            full_path.len() as u32,
            full_path.as_mut_ptr(),
            ptr::null_mut()
        )
    };

    if len == 0 {
        return Err(io::Error::last_os_error());
    }

    // 获取挂载点
    let mut mount_point = [0u16; 4096];
    let success = unsafe {
        winapi::GetVolumePathNameW(
            full_path.as_ptr(),
            mount_point.as_mut_ptr(),
            mount_point.len() as u32
        )
    };

    if success == 0 {
        return Err(io::Error::last_os_error());
    }

    from_wide_ptr(mount_point.as_ptr())
}

pub fn is_same_vol(path1: &str, path2: &str) -> bool {
    let mount_point1 = match get_volume_mount_point(path1) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let mount_point2 = match get_volume_mount_point(path2) {
        Ok(m) => m,
        Err(_) => return false,
    };

    MOUNT_POINTS
        .get(&mount_point1)
        .and_then(|d1| MOUNT_POINTS.get(&mount_point2).map(|d2| d1 == d2))
        .unwrap_or(false)
}
