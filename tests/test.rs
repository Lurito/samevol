#[cfg(test)]
mod test {
    use samevol::*;

    #[test]
    fn test_basic() {
        let path1 = r"C:\Windows\System32";
        let path2 = r"C:\Users\Public";
        assert!(is_same_vol(path1, path2));
    }

    #[test]
    fn test_vhd_detection() {
        let path1 = r"D:\";
        let path2 = r"D:\Vdisks\Wechat\another_file.txt"; // 假设这个目录为 vhd(x) 挂载点
        assert!(!is_same_vol(path1, path2));
    }

    #[test]
    fn test_resolve_device_path_of_relative() {
        let path = r"src";
        let resolved_path1 = resolve_device_path(path).unwrap();

        let current_dir = std::env::current_dir().unwrap();
        let current_dir_str = current_dir.to_str().unwrap();
        let resolved_path2 = resolve_device_path(current_dir_str).unwrap();

        assert_eq!(resolved_path1, resolved_path2);
    }
}
