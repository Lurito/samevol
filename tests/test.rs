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
}
