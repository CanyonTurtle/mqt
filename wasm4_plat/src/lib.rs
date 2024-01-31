#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;

mod kitty_ss;
mod title_ss;

use kittygame::multiplatform_defs::{BlitSubFlags, Spritesheet};
use kittygame::spritesheet::KITTY_SPRITESHEET_FLAGS;
use title_ss::OUTPUT_ONLINEPNGTOOLS_FLAGS;
use wasm4::*;

// pub type BlitSubFunc<'a> = dyn Fn(Spritesheet, i32, i32, u32, u32, u32, u32, BlitSubFlags) + 'a;
// pub type LineFunc<'a> = dyn Fn(i32, i32, i32, i32) + 'a;
// pub type TextStrFunc<'a> = dyn Fn(&str, i32, i32) + 'a;
// pub type RectFunc<'a> = dyn Fn(i32, i32, u32, u32) + 'a;
fn my_blit_sub(spritesehet: Spritesheet, x: i32, y: i32, w: u32, h: u32, src_x: u32, src_y: u32, flags: BlitSubFlags) {
    let mut bitflags = match spritesehet {
        Spritesheet::Main => KITTY_SPRITESHEET_FLAGS,
        Spritesheet::Title => OUTPUT_ONLINEPNGTOOLS_FLAGS,
    };
    
    if flags.flip_x {
        bitflags |= BLIT_FLIP_X
    }
    if flags.flip_y {
        bitflags |= BLIT_FLIP_Y
    }
    blit_sub(
        match spritesehet {
            Spritesheet::Main => kitty_ss::KITTY_SPRITESHEET,
            Spritesheet::Title => &title_ss::OUTPUT_ONLINEPNGTOOLS
        },
        x, y, w, h, src_x, src_y, 
        match spritesehet {
            Spritesheet::Main => kitty_ss::KITTY_SPRITESHEET_STRIDE as u32,
            Spritesheet::Title => title_ss::OUTPUT_ONLINEPNGTOOLS_WIDTH
        },
        bitflags
    )
}

fn my_line(x1: i32, y1: i32, x2: i32, y2: i32) {
    line(x1, y1, x2, y2);
}

fn my_rect(x1: i32, y1: i32, w: u32, h: u32) {
    rect(x1, y1, w, h);
}

fn my_text_str(t: &str, x: i32, y: i32) {
    text(t, x, y);
}

static mut PREVIOUS_GAMEPAD: [u8; 4] = [0, 0, 0, 0];

/// get joystick inputs from this and last frame.
fn get_inputs_this_frame() -> [[u8; 4]; 2] {
    let gamepads: [u8; 4] = unsafe { [*GAMEPAD1, *GAMEPAD2, *GAMEPAD3, *GAMEPAD4] };
    let mut btns_pressed_this_frame: [u8; 4] = [0; 4];

    for i in 0..gamepads.len() {
        let gamepad = gamepads[i];
        let previous = unsafe { PREVIOUS_GAMEPAD[i] };
        let pressed_this_frame = gamepad & (gamepad ^ previous);
        btns_pressed_this_frame[i] = pressed_this_frame;
    }
    unsafe { PREVIOUS_GAMEPAD.copy_from_slice(&gamepads[0..4]) };
    [btns_pressed_this_frame, gamepads]
}

#[no_mangle]
fn update() {
    let [btns_pressed_this_frame, gamepads] = get_inputs_this_frame();

    kittygame::kittygame_update(&my_blit_sub, &my_line, &my_rect, &my_text_str, 160, 160, &btns_pressed_this_frame, &gamepads);
}