/**
 * Copyright 2025 爱佐 (Ayrzo)
 *
 * This file is part of cargo crate samevol (https://docs.rs/samevol),
 * which licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

// 使用lazy_static初始化全局卷映射表
lazy_static::lazy_static! {
    /// 全局卷映射表，存储挂载点路径到卷设备路径的映射
    // 使用Arc<Mutex<>>实现线程安全访问
    static ref VOLUME_MAP: Arc<Mutex<HashMap<String, String>>> = {
        Arc::new(Mutex::new(
            // 初始化时构建卷映射表，失败时打印错误并返回空表
            build_volume_map().unwrap_or_else(|e| {
                eprintln!("Failed to initialize volume map: {}", e);
                HashMap::new()
            })
        ))
    };
}

/// Windows API FFI绑定模块
mod winapi {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        // 卷管理相关 API

        /// 查找第一个卷设备，返回搜索句柄
        ///
        /// # 参数
        /// - `lpsz_volume_name`: 接收卷名的缓冲区。缓冲区应至少为 MAX_PATH+1 宽字符
        /// - `cch_buffer_length`: 缓冲区大小（以宽字符计），包含终止空字符
        ///
        /// # 返回值
        /// - 成功时返回搜索句柄
        /// - 失败时返回 INVALID_HANDLE_VALUE
        ///
        /// # 安全性
        /// 需要确保缓冲区足够大并有效
        ///
        /// [微软文档](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstvolumew)
        pub fn FindFirstVolumeW(
            lpsz_volume_name: *mut u16,
            cch_buffer_length: u32,
        ) -> *mut std::ffi::c_void;

        /// 查找下一个卷设备
        ///
        /// # 参数
        /// - `h_find_volume`: 由 FindFirstVolumeW 返回的搜索句柄
        /// - `lpsz_volume_name`: 接收卷名的缓冲区
        /// - `cch_buffer_length`: 缓冲区大小（以宽字符计）
        ///
        /// # 返回值
        /// - 成功返回非零值
        /// - 失败返回 0（应调用 GetLastError 获取错误信息）
        ///
        /// # 安全性
        /// 需要确保句柄有效且缓冲区足够大
        ///
        /// [微软文档](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findnextvolumew)
        pub fn FindNextVolumeW(
            h_find_volume: *mut std::ffi::c_void,
            lpsz_volume_name: *mut u16,
            cch_buffer_length: u32,
        ) -> i32;

        /// 关闭卷搜索句柄
        ///
        /// # 参数
        /// - `h_find_volume`: 要关闭的搜索句柄
        ///
        /// # 返回值
        /// - 成功返回非零值
        /// - 失败返回 0
        ///
        /// # 安全性
        /// 需要确保句柄有效且未被重复关闭
        ///
        /// [微软文档](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findvolumeclose)
        pub fn FindVolumeClose(h_find_volume: *mut std::ffi::c_void) -> i32;

        /// 获取指定卷的所有挂载点路径
        ///
        /// # 参数
        /// - `lpsz_volume_name`: 输入卷名（GUID 格式），需以反斜杠结尾
        /// - `lpsz_volume_path_names`: 接收路径列表的缓冲区（多个以空字符分隔的路径）
        /// - `cch_buffer_length`: 缓冲区大小（以宽字符计）
        /// - `pcch_return_length`: 接收实际需要的缓冲区大小（不含终止符）
        ///
        /// # 返回值
        /// - 成功返回非零值
        /// - 失败返回 0（若缓冲区不足，会返回 ERROR_MORE_DATA）
        ///
        /// # 安全性
        /// 需要确保输入卷名格式正确，缓冲区足够大
        ///
        /// [微软文档](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumepathnamesforvolumenamew)
        pub fn GetVolumePathNamesForVolumeNameW(
            lpsz_volume_name: *const u16,
            lpsz_volume_path_names: *mut u16,
            cch_buffer_length: u32,
            pcch_return_length: *mut u32,
        ) -> i32;

        // 路径处理相关 API

        /// 获取文件完整路径（展开相对路径和环境变量）
        ///
        /// # 参数
        /// - `lp_file_name`: 输入路径（宽字符字符串）
        /// - `n_buffer_length`: 输出缓冲区大小（宽字符数）
        /// - `lp_buffer`: 接收完整路径的缓冲区
        /// - `lp_file_part`: 接收文件名部分起始位置的指针（可为 null）
        ///
        /// # 返回值
        /// - 成功返回复制到缓冲区的字符数（不含终止符）
        /// - 若缓冲区不足，返回所需缓冲区大小（含终止符）
        /// - 失败返回 0
        ///
        /// # 安全性
        /// 需要确保输入指针有效，缓冲区足够大
        ///
        /// [微软文档](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfullpathnamew)
        pub fn GetFullPathNameW(
            lp_file_name: *const u16,
            n_buffer_length: u32,
            lp_buffer: *mut u16,
            lp_file_part: *mut *mut u16,
        ) -> u32;

        /// 获取路径所属的卷挂载点
        ///
        /// # 参数
        /// - `lpsz_file_name`: 输入文件路径（宽字符字符串）
        /// - `lpsz_volume_path_name`: 输出挂载点路径的缓冲区
        /// - `cch_buffer_length`: 缓冲区大小（宽字符数）
        ///
        /// # 返回值
        /// - 成功返回非零值
        /// - 失败返回 0
        ///
        /// # 安全性
        /// 需要确保缓冲区足够大（通常至少 MAX_PATH 长度）
        ///
        /// [微软文档](https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getvolumepathnamew)
        pub fn GetVolumePathNameW(
            lpsz_file_name: *const u16,
            lpsz_volume_path_name: *mut u16,
            cch_buffer_length: u32,
        ) -> i32;
    }
}

/// Windows API调用结果类型别名
type WinResult<T> = Result<T, io::Error>;

/// 将Rust字符串转换为Windows宽字符字符串
fn wide_string(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt as _;

    OsStr::new(s)
        .encode_wide()  // 转换为 UTF-16 编码迭代器
        .chain(Some(0)) // 追加终止符
        .collect()      // collect as Vec<u16>
}

/// 从宽字符缓冲区读取终止字符串
fn from_wide_buf(buffer: &[u16]) -> WinResult<String> {
    // 找到第一个终止符的位置
    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    // 转换为UTF-8字符串
    String::from_utf16(&buffer[..end])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "UTF-16 conversion failed"))
}

/// 构建系统卷到挂载点路径的映射表
fn build_volume_map() -> WinResult<HashMap<String, String>> {
    let mut volume_map = HashMap::new();

    /* 卷名缓冲区说明：
     * 格式：`\\?\Volume{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}\`
     * 总长度：4(前缀`\\?\`) + 7(`Volume{`) + 36(GUID) + 2(`}\`) + 1(`\0`) = 50 个宽字符
     */
    let mut buffer = [0u16; 50];

    // 启动卷枚举
    let handle = unsafe { winapi::FindFirstVolumeW(buffer.as_mut_ptr(), buffer.len() as u32) };
    if handle.is_null() {
        return Err(io::Error::last_os_error());
    }

    // 遍历所有卷设备
    loop {
        // 转换当前卷名
        let volume_name = from_wide_buf(&buffer)?;

        // 准备路径缓冲区（4KiB）
        let mut paths_buffer = [0u16; 4096];
        let mut returned_len = 0;
        // 获取该卷的所有挂载点路径
        let success = unsafe {
            winapi::GetVolumePathNamesForVolumeNameW(
                buffer.as_ptr(),           // 输入卷名
                paths_buffer.as_mut_ptr(), // 输出路径列表
                paths_buffer.len() as u32, // 缓冲区大小
                &mut returned_len,         // 接收实际需要大小
            )
        };

        // 处理获取到的路径
        if success != 0 && returned_len > 0 {
            let mut offset = 0;
            // 遍历多重null终止的路径列表
            while offset < paths_buffer.len() {
                if paths_buffer[offset] == 0 {
                    break; // 遇到双重终止符，结束遍历
                }

                // 提取单个路径
                let end = paths_buffer[offset..]
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(paths_buffer.len() - offset);
                let path = from_wide_buf(&paths_buffer[offset..offset + end])?;

                // 规范化路径格式：统一使用反斜杠并确保结尾反斜杠
                let normalized_path = path.replace('/', "\\");
                let key = if !normalized_path.ends_with('\\') {
                    format!("{}\\", normalized_path)  // 追加反斜杠用于前缀匹配
                } else {
                    normalized_path
                };

                // 插入映射表（挂载点路径 -> 卷设备路径）
                volume_map.insert(key, volume_name.clone());

                offset += end + 1; // 移动到下一个路径
            }
        }

        // 获取下一个卷
        let next = unsafe {
            buffer.fill(0);  // 清空缓冲区
            winapi::FindNextVolumeW(handle, buffer.as_mut_ptr(), buffer.len() as u32)
        };
        if next == 0 {  // 枚举完成或出错
            break;
        }
    }

    // 关闭卷搜索句柄
    unsafe { winapi::FindVolumeClose(handle) };
    Ok(volume_map)
}

/// 获取给定路径所在的卷挂载点
fn get_volume_mount_point(path: &str) -> WinResult<String> {
    // 转换为宽字符路径
    let path_wide = wide_string(path);
    let mut full_path = [0u16; 4096];
    let mut mount_point = [0u16; 4096];

    // 第一步：获取绝对路径
    let len = unsafe {
        winapi::GetFullPathNameW(
            path_wide.as_ptr(),     // 输入路径
            full_path.len() as u32, // 输出缓冲区大小
            full_path.as_mut_ptr(), // 输出缓冲区
            std::ptr::null_mut(),        // 不需要文件名部分
        )
    };
    if len == 0 {
        return Err(io::Error::last_os_error());
    }

    // 第二步：获取挂载点路径
    let success = unsafe {
        winapi::GetVolumePathNameW(
            full_path.as_ptr(),       // 输入绝对路径
            mount_point.as_mut_ptr(), // 输出挂载点路径
            mount_point.len() as u32, // 缓冲区大小
        )
    };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }

    // 转换结果并确保以反斜杠结尾
    from_wide_buf(&mount_point).map(|s| {
        if s.ends_with('\\') { s } else { format!("{}\\", s) }
    })
}

// 重新初始化卷映射表
// 返回操作结果（成功包含映射数量，失败包含错误信息）
/// Re-initializes the volume mapping table by rebuilding it from the system.
///
/// # Returns
/// - `Ok(usize)`: Number of volume mappings found
/// - `Err(io::Error)`: Error encountered during rebuilding
///
/// # Notes
/// This will lock the global volume map mutex during update.
///
/// # Example
///
/// ```rust
/// use samevol::reinitialize_volume_map;
///
/// // After system storage configuration changes
/// let count = reinitialize_volume_map().expect("Failed to refresh mappings");
/// println!("Reloaded {} volume mappings", count);
/// ```
pub fn reinitialize_volume_map() -> Result<usize, io::Error> {
    let new_map = build_volume_map()?;
    let count = new_map.len();

    // 锁定并更新全局映射表
    let mut map = VOLUME_MAP.lock()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Mutex poison error: {}", e)))?;

    *map = new_map;
    Ok(count)
}

/// Resolves the device path of volume for a given file system path.
///
/// # Arguments
/// * `path` - The file system path to resolve (can be absolute or relative)
///
/// # Returns
/// - `Some(String)`: The device path in the format
///   `\\?\Volume{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}\`
/// - `None`: If the path cannot be resolved or the volume mapping is not found
///
/// # Errors
/// This function may return `None` in the following cases:
/// - The input path is invalid or inaccessible
/// - The volume map has not been properly initialized
/// - The path does not match any known mount points
///
/// # Example
/// ```rust
/// use samevol::resolve_device_path;
///
/// let path = r"C:\Windows\System32\drivers\etc\hosts";
/// let device_path = resolve_device_path(path).expect("Failed to resolve volume");
/// println!("Device path: {}", device_path);
/// ```
///
/// # Notes
/// - The function uses the global volume map initialized at startup
/// - For relative paths, the current working directory is used as the base
/// - The returned device path includes the `\\?\` prefix and trailing backslash
pub fn resolve_device_path(path: &str) -> Option<String> {
    // 获取挂载点路径
    let mount_point = match get_volume_mount_point(path) {
        Ok(m) => m,
        Err(_) => return None,
    };

    // 获取锁并访问映射表
    let map = VOLUME_MAP.lock().ok()?;

    // 查找所有可能的前缀匹配项
    let candidates = map.keys()
        .filter(|k| mount_point.starts_with(*k))
        .collect::<Vec<_>>();

    // 选择最长匹配的挂载点路径（最精确的父路径）
    let mount_path = candidates.iter()
        .max_by_key(|k| k.len())?;

    // 获取对应的设备路径
    map.get(*mount_path).cloned()
}

/// Checks if two paths reside on the same volume.
///
/// # Arguments
/// * `path1` - First path to check
/// * `path2` - Second path to check
///
/// # Returns
/// `true` if both paths are on the same volume, `false` otherwise (including error cases).
///
/// # Implementation Details
/// 1. Resolves each path's mount point
/// 2. Finds the longest matching mount point path in the volume map
/// 3. Compares the underlying device paths
///
/// # Example
/// ```rust
/// use samevol::is_same_vol;
///
/// let path1 = r"C:\Windows\System32";
/// let path2 = r"D:\Data\test.txt";
///
/// println!("Same volume? {}", is_same_vol(path1, path2)); // false
/// ```
pub fn is_same_vol(path1: &str, path2: &str) -> bool {
    // 比较两个路径所在卷的设备路径 (device path)
    let vol1 = resolve_device_path(path1);
    let vol2 = resolve_device_path(path2);

    vol1.zip(vol2).is_some_and(|(v1, v2)| v1 == v2)
}
