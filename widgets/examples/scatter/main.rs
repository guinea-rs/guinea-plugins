//! The scatter chart, which draws with WinUI: elsewhere it says so and
//! stops.

#[cfg(all(windows, feature = "winui"))]
mod demo;

fn main() {
    #[cfg(all(windows, feature = "winui"))]
    demo::run();

    #[cfg(not(all(windows, feature = "winui")))]
    eprintln!("the scatter example draws with WinUI, on Windows only");
}
