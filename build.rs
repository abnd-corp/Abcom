fn main() {
    println!("cargo:rerun-if-changed=assets/app_icon.ico");
    println!("cargo:rerun-if-changed=Cargo.toml");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/app_icon.ico")
            .set("ProductName", "Abcom")
            .set("FileDescription", "Abcom - LAN messenger")
            .compile()
            .expect("impossible d'intégrer les ressources Windows");
    }
}
