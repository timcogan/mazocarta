#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod app;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod combat;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod content;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod dungeon;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod rng;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod save;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use app::App;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static APP: RefCell<App> = RefCell::new(App::new());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_init(width: f32, height: f32) {
    APP.with(|app| app.borrow_mut().initialize(width, height));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_resize(width: f32, height: f32) {
    APP.with(|app| app.borrow_mut().resize(width, height));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_tick(dt_ms: f32) {
    APP.with(|app| app.borrow_mut().tick(dt_ms));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_mix_entropy(low: u32, high: u32) {
    APP.with(|app| app.borrow_mut().mix_entropy(low, high));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_debug_mode(enabled: u32) {
    APP.with(|app| app.borrow_mut().set_debug_mode(enabled != 0));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_saved_run_available(available: u32) {
    APP.with(|app| app.borrow_mut().set_saved_run_available(available != 0));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_language(code: u32) {
    APP.with(|app| {
        app.borrow_mut()
            .set_language(crate::content::Language::from_code(code))
    });
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_language_code() -> u32 {
    APP.with(|app| app.borrow().language_code())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_language_generation() -> u32 {
    APP.with(|app| app.borrow().language_generation())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_install_capability(code: u32) {
    APP.with(|app| {
        app.borrow_mut()
            .set_install_capability(crate::app::InstallCapability::from_code(code))
    });
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_update_available(available: u32) {
    APP.with(|app| app.borrow_mut().set_update_available(available != 0));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_is_boot_screen() -> u32 {
    APP.with(|app| u32::from(app.borrow().is_boot_screen()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pointer_down(x: f32, y: f32) {
    APP.with(|app| app.borrow_mut().pointer_down(x, y));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pointer_move(x: f32, y: f32) {
    APP.with(|app| app.borrow_mut().pointer_move(x, y));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pointer_up(x: f32, y: f32) {
    APP.with(|app| app.borrow_mut().pointer_up(x, y));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn key_down(key_code: u32) {
    APP.with(|app| app.borrow_mut().key_down(key_code));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn frame_ptr() -> *const u8 {
    APP.with(|app| app.borrow().frame_ptr())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn frame_len() -> usize {
    APP.with(|app| app.borrow().frame_len())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn run_save_generation() -> u32 {
    APP.with(|app| app.borrow().run_save_generation())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn run_save_ptr() -> *const u8 {
    APP.with(|app| app.borrow().run_save_ptr())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn run_save_len() -> usize {
    APP.with(|app| app.borrow().run_save_len())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn resume_request_pending() -> u32 {
    APP.with(|app| u32::from(app.borrow().resume_request_pending()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn clear_resume_request() {
    APP.with(|app| app.borrow_mut().clear_resume_request());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn install_request_pending() -> u32 {
    APP.with(|app| u32::from(app.borrow().install_request_pending()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn clear_install_request() {
    APP.with(|app| app.borrow_mut().clear_install_request());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn update_request_pending() -> u32 {
    APP.with(|app| u32::from(app.borrow().update_request_pending()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn clear_update_request() {
    APP.with(|app| app.borrow_mut().clear_update_request());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn prepare_restore_buffer(len: usize) -> *mut u8 {
    APP.with(|app| app.borrow_mut().prepare_restore_buffer(len))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_restore_from_buffer(len: usize) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().restore_from_buffer(len)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn share_request_ptr() -> *const u8 {
    APP.with(|app| app.borrow().share_request_ptr())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn share_request_len() -> usize {
    APP.with(|app| app.borrow().share_request_len())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn clear_share_request() {
    APP.with(|app| app.borrow_mut().clear_share_request());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn share_capture_x() -> f32 {
    APP.with(|app| app.borrow().share_capture_x())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn share_capture_y() -> f32 {
    APP.with(|app| app.borrow().share_capture_y())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn share_capture_w() -> f32 {
    APP.with(|app| app.borrow().share_capture_w())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn share_capture_h() -> f32 {
    APP.with(|app| app.borrow().share_capture_h())
}
