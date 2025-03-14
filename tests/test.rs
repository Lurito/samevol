#[cfg(test)]
mod test {
    use samevol::*;

    #[test]
    fn test_basic() {
        let path1 = r"C:\Windows\System32";
        let path2 = r"C:\Users\Public";
        assert!(is_same_vol(path1, path2));
    }
}
