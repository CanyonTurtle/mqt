


pub const BUTTON_1: u8 = 1;
pub const BUTTON_2: u8 = 2;
pub const BUTTON_LEFT: u8 = 16;
pub const BUTTON_RIGHT: u8 = 32;
pub const BUTTON_UP: u8 = 64;
pub const BUTTON_DOWN: u8 = 128;


/// Which spritesheet to render with.
pub enum Spritesheet {
    Main, // pull from the main spritesheet.
    Title, // pull from the title 
}

pub struct BlitSubFlags {
    pub flip_x: bool,
    pub flip_y: bool,
}

pub type BlitSubFunc<'a> = dyn Fn(Spritesheet, i32, i32, u32, u32, u32, u32, BlitSubFlags) + 'a;
pub type LineFunc<'a> = dyn Fn(i32, i32, i32, i32) + 'a;
pub type TextStrFunc<'a> = dyn Fn(&str, i32, i32) + 'a;
pub type RectFunc<'a> = dyn Fn(i32, i32, u32, u32) + 'a;
