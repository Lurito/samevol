fn main() {
    #[cfg(not(target_os = "windows"))]
    panic!("This library only supports Windows.");
}
