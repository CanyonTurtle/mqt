


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

// define the colors of a Pallette.
pub struct Pallette {
    pub main_kitty: u32,
    pub pigs_lizards: u32,
    pub foreground: u32,
    pub background: u32,
}

pub enum DrawColor {
    MainKitty,
    PigsLizards,
    Foreground,
    Background,
}

pub type BlitSubFunc<'a> = dyn FnMut(Spritesheet, i32, i32, u32, u32, u32, u32, BlitSubFlags) + 'a;
pub type LineFunc<'a> = dyn FnMut(i32, i32, i32, i32, &DrawColor) + 'a;
pub type TextStrFunc<'a> = dyn FnMut(&str, i32, i32, &DrawColor) + 'a;
pub type RectFunc<'a> = dyn FnMut(i32, i32, u32, u32, &DrawColor) + 'a;
pub type SwitchPalletteFunc<'a> = dyn FnMut(&Pallette) + 'a;

