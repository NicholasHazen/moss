mod playback;

#[cfg(target_arch = "wasm32")]
mod browser;
mod view;

#[cfg(target_arch = "wasm32")]
fn main() {
    browser::run();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Moss is a browser app. From the workspace root, run: trunk serve --locked");
}
