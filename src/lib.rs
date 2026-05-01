#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod app;
#[cfg(any(not(target_arch = "wasm32"), feature = "e2e"))]
mod autoplay;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod combat;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod content;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod dungeon;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod pairing;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod party;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod rng;
mod run_logic;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod save;
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod session;
#[cfg(not(target_arch = "wasm32"))]
pub mod sim;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use app::App;
#[cfg(target_arch = "wasm32")]
use pairing::{PairingBridge, TransportSubmitResult};

#[cfg(target_arch = "wasm32")]
thread_local! {
    static APP: RefCell<App> = RefCell::new(App::new());
    static PAIRING: RefCell<PairingBridge> = RefCell::new(PairingBridge::default());
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
pub extern "C" fn app_set_background_mode(code: u32) {
    APP.with(|app| {
        app.borrow_mut()
            .set_background_mode(crate::app::BackgroundMode::from_code(code))
    });
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn prepare_pairing_buffer(len: usize) -> *mut u8 {
    PAIRING.with(|pairing| pairing.borrow_mut().prepare_input_buffer(len))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_output_ptr() -> *const u8 {
    PAIRING.with(|pairing| pairing.borrow().output_ptr())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_output_len() -> usize {
    PAIRING.with(|pairing| pairing.borrow().output_len())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_decoded_payload_ptr() -> *const u8 {
    PAIRING.with(|pairing| pairing.borrow().decoded_payload_ptr())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_decoded_payload_len() -> usize {
    PAIRING.with(|pairing| pairing.borrow().decoded_payload_len())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_encode_payload_from_buffer(len: usize) -> u32 {
    PAIRING.with(|pairing| u32::from(pairing.borrow_mut().encode_payload_from_buffer(len).is_ok()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_decode_payload_from_buffer(len: usize) -> u32 {
    PAIRING.with(|pairing| u32::from(pairing.borrow_mut().decode_payload_from_buffer(len).is_ok()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_build_transport_frames_from_buffer(len: usize, chunk_chars: u32) -> u32 {
    PAIRING.with(|pairing| {
        pairing
            .borrow_mut()
            .build_transport_frames_from_buffer(len, chunk_chars as usize)
            .map(|count| count as u32)
            .unwrap_or(0)
    })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_transport_frame_count() -> usize {
    PAIRING.with(|pairing| pairing.borrow().transport_frame_count())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_export_transport_frame(index: usize) -> u32 {
    PAIRING.with(|pairing| u32::from(pairing.borrow_mut().export_transport_frame(index)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_reset_transport_assembly() {
    PAIRING.with(|pairing| pairing.borrow_mut().reset_transport_assembly());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_submit_transport_text_from_buffer(len: usize) -> u32 {
    PAIRING.with(
        |pairing| match pairing.borrow_mut().submit_transport_text_from_buffer(len) {
            Ok(TransportSubmitResult::DirectCode) => 1,
            Ok(TransportSubmitResult::Partial) => 2,
            Ok(TransportSubmitResult::Complete) => 3,
            Err(_) => 0,
        },
    )
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_transport_received_parts() -> usize {
    PAIRING.with(|pairing| pairing.borrow().transport_received_parts())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn pairing_transport_total_parts() -> usize {
    PAIRING.with(|pairing| pairing.borrow().transport_total_parts())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_background_mode_code() -> u32 {
    APP.with(|app| app.borrow().background_mode_code())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_background_mode_generation() -> u32 {
    APP.with(|app| app.borrow().background_mode_generation())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn prepare_player_name_buffer(len: usize) -> *mut u8 {
    APP.with(|app| app.borrow_mut().prepare_player_name_buffer(len))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_player_name_from_buffer(len: usize) -> bool {
    APP.with(|app| app.borrow_mut().set_player_name_from_buffer(len))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn prepare_party_slot_name_buffer(len: usize) -> *mut u8 {
    APP.with(|app| app.borrow_mut().prepare_party_slot_name_buffer(len))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_sync_remote_party_slot_from_buffer(
    slot: u32,
    connected: u32,
    ready: u32,
    in_run: u32,
    in_combat: u32,
    alive: u32,
    hp: i32,
    max_hp: i32,
    block: i32,
    len: usize,
) -> bool {
    APP.with(|app| {
        app.borrow_mut().sync_remote_party_slot_from_buffer(
            slot,
            connected != 0,
            ready != 0,
            in_run != 0,
            in_combat != 0,
            alive != 0,
            hp,
            max_hp,
            block,
            len,
        )
    })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_disconnect_remote_party_slot(slot: u32) -> bool {
    APP.with(|app| app.borrow_mut().disconnect_remote_party_slot(slot))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_clear_remote_party_slot(slot: u32) -> bool {
    APP.with(|app| app.borrow_mut().clear_remote_party_slot(slot))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_player_name_generation() -> u32 {
    APP.with(|app| app.borrow().player_name_generation())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_player_name_ptr() -> *const u8 {
    APP.with(|app| app.borrow().player_name_ptr())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_player_name_len() -> usize {
    APP.with(|app| app.borrow().player_name_len())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_player_name_input_focused(focused: u32) {
    APP.with(|app| {
        app.borrow_mut().set_player_name_input_focused(focused != 0);
    });
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_settings_player_name_input_visible() -> u32 {
    APP.with(|app| u32::from(app.borrow().settings_player_name_input_visible()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_settings_player_name_input_x() -> f32 {
    APP.with(|app| app.borrow().settings_player_name_input_x())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_settings_player_name_input_y() -> f32 {
    APP.with(|app| app.borrow().settings_player_name_input_y())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_settings_player_name_input_w() -> f32 {
    APP.with(|app| app.borrow().settings_player_name_input_w())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_settings_player_name_input_h() -> f32 {
    APP.with(|app| app.borrow().settings_player_name_input_h())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_settings_player_name_input_font_size() -> f32 {
    APP.with(|app| app.borrow().settings_player_name_input_font_size())
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
pub extern "C" fn app_module_select_card_index_at_point(x: f32, y: f32) -> i32 {
    APP.with(|app| app.borrow().module_select_card_index_at_point(x, y))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_continue_button_hit_at_point(x: f32, y: f32) -> u32 {
    APP.with(|app| u32::from(app.borrow().continue_button_hit_at_point(x, y)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_reward_card_index_at_point(x: f32, y: f32) -> i32 {
    APP.with(|app| app.borrow().reward_card_index_at_point(x, y))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_reward_skip_hit_at_point(x: f32, y: f32) -> u32 {
    APP.with(|app| u32::from(app.borrow().reward_skip_hit_at_point(x, y)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_event_choice_index_at_point(x: f32, y: f32) -> i32 {
    APP.with(|app| app.borrow().event_choice_index_at_point(x, y))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_finish_opening_intro() -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_opening_intro_action()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_continue_level_intro() -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_level_intro_action()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_claim_module_select(index: u32) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_module_select_card(index as usize)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_claim_reward(index: u32) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_reward_card(index as usize)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_skip_reward() -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_reward_skip()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_claim_event_choice(index: u32) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_event_choice(index as usize)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_claim_rest_heal() -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_rest_heal()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_claim_rest_upgrade(index: u32) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_rest_upgrade(index as usize)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_claim_shop_offer(index: u32) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_shop_offer(index as usize)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_leave_shop() -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().activate_shop_leave()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_multiplayer_combat_action_code_at_point(x: f32, y: f32) -> u32 {
    APP.with(|app| app.borrow().multiplayer_combat_action_code_at_point(x, y))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_multiplayer_combat_action_code_for_key(key_code: u32) -> u32 {
    APP.with(|app| {
        app.borrow()
            .multiplayer_combat_action_code_for_key(key_code)
    })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_handle_local_multiplayer_combat_pointer_down(x: f32, y: f32) -> u32 {
    APP.with(|app| {
        app.borrow_mut()
            .handle_local_multiplayer_combat_pointer_down(x, y)
    })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_handle_local_multiplayer_combat_key(key_code: u32) -> u32 {
    APP.with(|app| {
        app.borrow_mut()
            .handle_local_multiplayer_combat_key(key_code)
    })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_menu_button_hit_at_point(x: f32, y: f32) -> u32 {
    APP.with(|app| u32::from(app.borrow().menu_button_hit_at_point(x, y)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_apply_multiplayer_combat_action_code(slot: u32, code: u32) -> u32 {
    APP.with(|app| {
        u32::from(
            app.borrow_mut()
                .apply_multiplayer_combat_action_code(slot as usize, code),
        )
    })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_apply_local_multiplayer_combat_action_code(code: u32) -> u32 {
    APP.with(|app| {
        u32::from(
            app.borrow_mut()
                .apply_local_multiplayer_combat_action_code(code),
        )
    })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_clear_local_multiplayer_pending_combat_action() -> u32 {
    APP.with(|app| {
        app.borrow_mut()
            .clear_local_multiplayer_pending_combat_action();
        1
    })
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
pub extern "C" fn enemy_sprite_width(code: u32) -> u32 {
    crate::content::enemy_sprite_layer_def_by_code(code as u8)
        .map(|sprite| sprite.width as u32)
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn enemy_sprite_height(code: u32) -> u32 {
    crate::content::enemy_sprite_layer_def_by_code(code as u8)
        .map(|sprite| sprite.height as u32)
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn enemy_sprite_data_ptr(code: u32) -> *const u8 {
    crate::content::enemy_sprite_layer_def_by_code(code as u8)
        .map(|sprite| sprite.bits.as_ptr())
        .unwrap_or(std::ptr::null())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn enemy_sprite_data_len(code: u32) -> usize {
    crate::content::enemy_sprite_layer_def_by_code(code as u8)
        .map(|sprite| sprite.bits.len())
        .unwrap_or(0)
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
pub extern "C" fn app_begin_remote_input_slot(slot: u32) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().begin_remote_input_slot(slot)))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_end_remote_input_slot() {
    APP.with(|app| app.borrow_mut().end_remote_input_slot());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn party_snapshot_generation() -> u32 {
    APP.with(|app| app.borrow().party_snapshot_generation())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn party_snapshot_ptr() -> *const u8 {
    APP.with(|app| app.borrow().party_snapshot_ptr())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn party_snapshot_len() -> usize {
    APP.with(|app| app.borrow().party_snapshot_len())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_party_size() -> u32 {
    APP.with(|app| app.borrow().party_size())
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_set_party_size(size: u32) {
    APP.with(|app| app.borrow_mut().set_party_size(size));
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_start_multiplayer_run() -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().start_multiplayer_run_from_web()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_resume_multiplayer_run() -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().resume_multiplayer_run_from_web()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_return_to_menu() -> u32 {
    APP.with(|app| {
        app.borrow_mut().return_to_menu();
        1
    })
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_load_e2e_fixture(code: u32) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().load_e2e_fixture(code)))
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_set_next_run_seed(low: u32, high: u32) {
    APP.with(|app| {
        app.borrow_mut()
            .debug_set_next_run_seed(((high as u64) << 32) | low as u64)
    });
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_autoplay_action_code() -> u32 {
    APP.with(|app| app.borrow().debug_autoplay_action_code())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_autoplay_action_param_a() -> u32 {
    APP.with(|app| app.borrow().debug_autoplay_action_param_a())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_autoplay_action_param_b() -> u32 {
    APP.with(|app| app.borrow().debug_autoplay_action_param_b())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_apply_autoplay_action(code: u32, param_a: u32, param_b: u32) -> u32 {
    APP.with(|app| {
        u32::from(
            app.borrow_mut()
                .debug_apply_autoplay_action(code, param_a, param_b),
        )
    })
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
pub extern "C" fn multiplayer_request_pending() -> u32 {
    APP.with(|app| u32::from(app.borrow().multiplayer_request_pending()))
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn clear_multiplayer_request() {
    APP.with(|app| app.borrow_mut().clear_multiplayer_request());
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn app_apply_multiplayer_snapshot_from_buffer(len: usize) -> u32 {
    APP.with(|app| u32::from(app.borrow_mut().apply_multiplayer_snapshot_from_buffer(len)))
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_combat_hint_code() -> u32 {
    APP.with(|app| app.borrow().debug_combat_hint_code())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_combat_input_locked() -> u32 {
    APP.with(|app| u32::from(app.borrow().debug_combat_input_locked()))
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_combat_lock_mask() -> u32 {
    APP.with(|app| app.borrow().debug_combat_lock_mask())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_pending_local_multiplayer_combat_action_code() -> u32 {
    APP.with(|app| {
        app.borrow()
            .debug_pending_local_multiplayer_combat_action_code()
    })
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_combat_playback_queue_len() -> u32 {
    APP.with(|app| app.borrow().debug_combat_playback_queue_len())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_combat_active_stat_count() -> u32 {
    APP.with(|app| app.borrow().debug_combat_active_stat_count())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_debug_result_code() -> u32 {
    APP.with(|app| app.borrow().debug_result_code())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_hand_len() -> u32 {
    APP.with(|app| app.borrow().combat_hand_len_for_debug())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_enemy_count() -> u32 {
    APP.with(|app| app.borrow().combat_enemy_count_for_debug())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_hand_card_center_x(index: u32) -> f32 {
    APP.with(|app| app.borrow().combat_hand_card_center_x(index))
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_hand_card_center_y(index: u32) -> f32 {
    APP.with(|app| app.borrow().combat_hand_card_center_y(index))
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_enemy_center_x(index: u32) -> f32 {
    APP.with(|app| app.borrow().combat_enemy_center_x(index))
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_enemy_center_y(index: u32) -> f32 {
    APP.with(|app| app.borrow().combat_enemy_center_y(index))
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_player_center_x() -> f32 {
    APP.with(|app| app.borrow().combat_player_center_x())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_player_center_y() -> f32 {
    APP.with(|app| app.borrow().combat_player_center_y())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_end_turn_center_x() -> f32 {
    APP.with(|app| app.borrow().combat_end_turn_center_x())
}

#[cfg(all(target_arch = "wasm32", feature = "e2e"))]
#[unsafe(no_mangle)]
pub extern "C" fn app_combat_end_turn_center_y() -> f32 {
    APP.with(|app| app.borrow().combat_end_turn_center_y())
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
