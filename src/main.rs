#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;

use ui::CouponApp;

fn main() -> eframe::Result<()> {
    // Configure the native window
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([420.0, 260.0])    // width x height in pixels
            .with_resizable(true),
        ..Default::default()
    };

    // Launch the app
    eframe::run_native(
        "Coupon Generator",
        options,
        Box::new(|_cc| Ok(Box::new(CouponApp::default()))),
    )
}
