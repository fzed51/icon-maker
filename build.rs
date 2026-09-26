// Intègre l'icône dans l'exe : c'est elle qu'affiche l'Explorateur.
fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set("FileDescription", "Icon Maker — crée ton icône");
        res.set("ProductName", "Icon Maker");
        res.compile()
            .expect("impossible d'intégrer l'icône de l'application");
    }
}
