fn main() {
    // Check if the target OS is Windows, but allow docs.rs for building docs
    if !cfg!(target_os = "windows") && std::env::var("DOCS_RS").is_err() {
        panic!("This library only supports Windows.");
    }
}
