#[cfg(feature = "gui")]
fn main() {
    tauri_build::build()
}

#[cfg(not(feature = "gui"))]
fn main() {}
