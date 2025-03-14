use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref VOLUME_MAP: Arc<Mutex<HashMap<String, String>>> = {
        Arc::new(Mutex::new(
            build_volume_map().unwrap_or_else(|e| {
                eprintln!("Failed to initialize volume map: {}", e);
                HashMap::new()
            })
        ))
    };
}

mod winapi {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        // Volume Management API
        pub fn FindFirstVolumeW(
            lpsz_volume_name: *mut u16,
            cch_buffer_length: u32,
        ) -> *mut std::ffi::c_void;

        pub fn FindNextVolumeW(
            h_find_volume: *mut std::ffi::c_void,
            lpsz_volume_name: *mut u16,
            cch_buffer_length: u32,
        ) -> i32;

        pub fn FindVolumeClose(h_find_volume: *mut std::ffi::c_void) -> i32;

        pub fn GetVolumePathNamesForVolumeNameW(
            lpsz_volume_name: *const u16,
            lpsz_volume_path_names: *mut u16,
            cch_buffer_length: u32,
            pcch_return_length: *mut u32,
        ) -> i32;

        // Path Resolution API
        pub fn GetFullPathNameW(
            lp_file_name: *const u16,
            n_buffer_length: u32,
            lp_buffer: *mut u16,
            lp_file_part: *mut *mut u16,
        ) -> u32;

        pub fn GetVolumePathNameW(
            lpsz_file_name: *const u16,
            lpsz_volume_path_name: *mut u16,
            cch_buffer_length: u32,
        ) -> i32;
    }
}

type WinResult<T> = Result<T, io::Error>;

fn wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn from_wide_buf(buffer: &[u16]) -> WinResult<String> {
    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16(&buffer[..end])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "UTF-16 conversion failed"))
}

fn build_volume_map() -> WinResult<HashMap<String, String>> {
    let mut volume_map = HashMap::new();
    
    /* 
     * 卷名的GUID格式为`\\?\Volume{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}\`，
     * 其中：
     * - `\\?\` 前缀占4字符
     * - `Volume{` 固定部分占7字符
     * - GUID部分占36字符
     * - `}\` 后缀占2字符
     * - Null终止符占1字符
     * 总计：4 + 7 + 36 + 2 + 1 = 50个宽字符（u16）。
     * 因此，缓冲区大小必须至少为50，以完整存储卷名。
     */
    let mut buffer = [0u16; 50]; 

    // 开始卷枚举
    let handle = unsafe { winapi::FindFirstVolumeW(buffer.as_mut_ptr(), buffer.len() as u32) };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }

    loop {
        let volume_name = from_wide_buf(&buffer)?;

        // 获取卷的挂载点路径
        let mut paths_buffer = [0u16; 4096];
        let mut returned_len = 0;
        let success = unsafe {
            winapi::GetVolumePathNamesForVolumeNameW(
                buffer.as_ptr(),
                paths_buffer.as_mut_ptr(),
                paths_buffer.len() as u32,
                &mut returned_len,
            )
        };

        if success != 0 && returned_len > 0 {
            let mut offset = 0;
            while offset < paths_buffer.len() {
                if paths_buffer[offset] == 0 {
                    break;
                }

                let end = paths_buffer[offset..]
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(paths_buffer.len() - offset);
                let path = from_wide_buf(&paths_buffer[offset..offset + end])?;

                // 规范化路径格式（统一反斜杠）
                let normalized_path = path.replace('/', "\\");
                if !normalized_path.ends_with('\\') {
                    volume_map.insert(format!("{}\\", normalized_path), volume_name.clone());
                } else {
                    volume_map.insert(normalized_path, volume_name.clone());
                }

                offset += end + 1;
            }
        }

        // 继续枚举下一个卷
        let next = unsafe {
            buffer.fill(0);
            winapi::FindNextVolumeW(handle, buffer.as_mut_ptr(), buffer.len() as u32)
        };
        if next == 0 {
            break;
        }
    }

    unsafe { winapi::FindVolumeClose(handle) };
    Ok(volume_map)
}

fn get_volume_mount_point(path: &str) -> WinResult<String> {
    let path_wide = wide_string(path);
    let mut full_path = [0u16; 4096];
    let mut mount_point = [0u16; 4096];

    // 获取绝对路径
    let len = unsafe {
        winapi::GetFullPathNameW(
            path_wide.as_ptr(),
            full_path.len() as u32,
            full_path.as_mut_ptr(),
            ptr::null_mut(),
        )
    };
    if len == 0 {
        return Err(io::Error::last_os_error());
    }

    // 获取挂载点路径
    let success = unsafe {
        winapi::GetVolumePathNameW(
            full_path.as_ptr(),
            mount_point.as_mut_ptr(),
            mount_point.len() as u32,
        )
    };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }

    from_wide_buf(&mount_point).map(|s| {
        if s.ends_with('\\') { s } else { format!("{}\\", s) }
    })
}

/// 重新初始化卷映射表
/// 返回操作结果（成功包含映射数量，失败包含错误信息）
pub fn reinitialize_volume_map() -> Result<usize, io::Error> {
    let new_map = build_volume_map()?;
    let count = new_map.len();

    // 获取锁并替换映射
    let mut map = VOLUME_MAP.lock()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Mutex poison error: {}", e)))?;

    *map = new_map;
    Ok(count)
}

pub fn is_same_vol(path1: &str, path2: &str) -> bool {
    let resolve_volume = |path: &str| -> Option<String> {
        let mount_point = match get_volume_mount_point(path) {
            Ok(m) => m,
            Err(_) => return None,
        };

        // 获取锁并访问映射表
        let map = VOLUME_MAP.lock().ok()?;

        // 查找最长匹配的挂载点路径
        let candidates = map.keys()
            .filter(|k| mount_point.starts_with(*k))
            .collect::<Vec<_>>();
        let mount_path = candidates.iter()
            .max_by_key(|k| k.len())?;

        // 获取对应的卷名
        map.get(*mount_path).cloned()
    };

    let vol1 = resolve_volume(path1);
    let vol2 = resolve_volume(path2);

    vol1.zip(vol2).is_some_and(|(v1, v2)| v1 == v2)
}
