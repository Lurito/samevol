fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os != "windows" {
        panic!("This library only supports Windows.");
    }
}
