use std::{cell::RefCell, collections::HashMap};

use macroquad::prelude::*;

use kittygame::{kittygame_update, multiplatform_defs::{BlitSubFlags, DrawColor, Pallette, Spritesheet, BUTTON_1, BUTTON_2, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP}};


const KITTY_SS_COLORS: [[u8; 4]; 5] = [
    [0xee, 0xc3, 0x9a, 0xff], // main kitty color
    [0xff, 0x67, 0xd3, 0xff], // pig / lizard color
    [0xff, 0xff, 0xff, 0xff], // foreground (tiles, cards)
    [0x12, 0x34, 0x56, 0x78], // background color (unused color on the spriteheet)
    [0x00, 0x00, 0x00, 0x00], // No color drawn on spritesheet (transparent)
];

const DEFAULT_COLOR_PALLETTE: [Color; 5] = [
    color_u8!(0xf8, 0xff, 0xd2, 0xff), // main kitty color
    color_u8!(0xff, 0x66, 0x33, 0xff), // lizard / pig color
    color_u8!(0xe4, 0xf2, 0x88, 0xff), // foreground (tiles, cards)
    color_u8!(0x57, 0xda, 0xb2, 0xff), // background / default
    color_u8!(0x00, 0x00, 0x00, 0x00), // transparent
];

const TITLE_COLOR_PALLETE: [[u8; 4]; 5] = [
    [0xF9, 0xDF, 0xD1, 0xff], // main letter color
    [0xEB, 0x9F, 0x9E, 0xff], // letter backing color
    [0x12, 0x00, 0x00, 0x00], // transparent (unused)
    [0x34, 0x00, 0x00, 0x00], // transparent (unused)
    [0x00, 0x00, 0x00, 0x00], // transparent
];

/// Convert colors from src to mapped, when they occur in an image.
fn build_colormap(src_colors: [[u8; 4]; 5], mapped_colors: [Color; 5]) -> HashMap<[u8; 4], Color> {
    let mut colormap = HashMap::new();

    for i in 0..src_colors.len() {
        colormap.insert(src_colors[i], mapped_colors[i]);
    }

    colormap
}

/// Create a new image with the colors replaced from some colormap.
fn recolor_spritesheet(image: &Image, colormap: HashMap<[u8; 4], Color>) -> Image {
    let mut im = image.clone();
    for pixel in im.get_image_data_mut().iter_mut(){
        let color: [u8; 4] = pixel.clone().into();
        *pixel = colormap[&color].into();
    }
    im
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Window name".to_owned(),
        window_width: 1280,
        window_height: 720,
        fullscreen: false,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let color_palette = RefCell::new(DEFAULT_COLOR_PALLETTE);

    set_pc_assets_folder("assets");

    const MAX_SCREEN_DIM: f32 = 400.;

    // let kitty_bg_texture: Texture2D = load_texture("kittygame.png").await.unwrap();
    let kitty_ss_texture: Texture2D = load_texture("kitty-ss.png").await.unwrap();
    let kitty_title_texture: Texture2D = load_texture("kitty_title.png").await.unwrap();

    // let font = load_ttf_font("SRAFreePixelFontPack/PixelSmall.ttf")

    // const FONT_HEIGHT: u16 = 9;
    // let mut font = load_ttf_font("Pixeloid_Font_0_5/TrueType (.ttf)/PixeloidSans.ttf")

    const FONT_HEIGHT: u16 = 8;

    let mut font = load_ttf_font("PressStart2P-Regular.ttf")
        .await
        .unwrap();

    font.set_filter(FilterMode::Nearest);

    let original_image = kitty_ss_texture.get_texture_data();

    let recolored_ss = recolor_spritesheet(&original_image, build_colormap(KITTY_SS_COLORS, *color_palette.borrow()));
    kitty_ss_texture.update(&recolored_ss);
    
    let recolored_title = recolor_spritesheet(&kitty_title_texture.get_texture_data(), build_colormap(TITLE_COLOR_PALLETE, DEFAULT_COLOR_PALLETTE));
    kitty_title_texture.update(&recolored_title);

    

    // kitty_bg_texture.set_filter(FilterMode::Nearest);
    kitty_ss_texture.set_filter(FilterMode::Nearest);
    kitty_title_texture.set_filter(FilterMode::Nearest);

    // we only want to create a new texture (and image) when necessary, because
    // it's expensive. So track when the screen changes dimensions.
    let mut last_sh = screen_height();
    let mut last_sw = screen_width();

    let mut bg_color;
    {
        let cp = &color_palette.borrow();
        bg_color = cp[cp.len() - 2];
    }

    

    // let mut fps = 0;
    let mut i = 0;
    loop {
        i += 1;
        if i % 15 == 0 {
            // fps = get_fps();
            i = 0;
            
        }
        // we want the dimensions of the screen to be:
        // minimum internal dimension is 160
        // maximum dimension is as large as will fill the screen

        let min_internal_dim = 160;

        let sh = screen_height();
        let sw = screen_width();

        if last_sh != sh || last_sw != sw {
            last_sh = sh;
            last_sw = sw;
            // create new texture
        }

        enum Dim {
            Width,
            Height,
        }

        let smaller_side = match sh <= sw {
            true => Dim::Height,
            _ => Dim::Width,
        };
        let smaller_real_dim = sh.min(sw);
        let larger_real_dim = sh.max(sw);

        let dim_ratio = min_internal_dim as f32 / smaller_real_dim as f32;

        let other_internal_dim = (larger_real_dim * dim_ratio).min(MAX_SCREEN_DIM) as i32;

        let internal_width;
        let internal_height;

        match smaller_side {
            Dim::Height => {
                internal_height = min_internal_dim;
                internal_width = other_internal_dim;
            }
            _ => {
                internal_width = min_internal_dim;
                internal_height = other_internal_dim;
            }
        }

        clear_background(bg_color);

        let mut cam = Camera2D::from_display_rect(Rect::new(
            0.,
            0.,
            internal_width as f32,
            internal_height as f32,
        ));

        cam.rotation = 180.;
        cam.zoom = Vec2::new(-1. * cam.zoom.x, cam.zoom.y);
        {
            set_camera(&cam);
        }

        // draw_texture(&kitty_title_texture, 10., 30., WHITE);

        // draw_texture_ex(
        //     &kitty_ss_texture,
        //     10.,
        //     80.,
        //     WHITE,
        //     DrawTextureParams {
        //         source: Some(Rect::new(16., 56., 16., 8.)),
        //         ..Default::default()
        //     },
        // );
        
        
        // draw_text_ex(
        //     &format!["{}", fps],
        //     10.,
        //     10.,
        //     TextParams {
        //         font_size: 9,
        //         font: Some(&font),
        //         font_scale: 1.,
        //         color: DEFAULT_COLOR_PALLETTE[0],
        //         ..Default::default()
        //     },
        // );

        // for i in 0..5 {
        //     for j in 0..5 {
        //         let (x, y) = (10. + i as f32 * 60., 20. + j as f32 * 60.);
        //         draw_text_ex(
        //             &format!["({}, {})", x, y],
        //             x,
        //             y,
        //             TextParams {
        //                 font_size: 9,
        //                 font: Some(&font),
        //                 font_scale: 1.,
        //                 color: DEFAULT_COLOR_PALLETTE[0],
        //                 ..Default::default()
        //             },
        //         );
        //     }
        // }
        
        //pub type BlitSubFunc = fn(Spritesheet, i32, i32, u32, u32, u32, u32, u32, BlitSubFlags);

        let blit_sub = |spritesheet: Spritesheet, x: i32, y: i32, w: u32, h: u32, src_x: u32, src_y: u32, flags: BlitSubFlags| {
            draw_texture_ex(
                match spritesheet {
                    Spritesheet::Main => &kitty_ss_texture,
                    Spritesheet::Title => &kitty_title_texture,
                },
                x as f32,
                y as f32,
                WHITE,
                DrawTextureParams{
                    source: Some(Rect{
                        x: src_x as f32,
                        y: src_y as f32,
                        w: w as f32,
                        h: h as f32
                    }),
                    flip_x: flags.flip_x,
                    flip_y: flags.flip_y,
                    ..Default::default()
                }
            )
        };

        fn map_pallete_color(color: &DrawColor) -> usize {
            match color {
                DrawColor::MainKitty => 0,
                DrawColor::PigsLizards => 1,
                DrawColor::Foreground => 2,
                DrawColor::Background => 3
            }
        }

        let line = |x1i: i32, y1i: i32, x2i: i32, y2i: i32, color: &DrawColor| {
            let (mut x1, mut y1, mut x2, mut y2) = (x1i as f32, y1i as f32, x2i as f32, y2i as f32);
            if x1 == x2 {
                x1 += 0.5;
                x2 += 0.5;
                // if line is being used as a pixel tool
                y2 += 1.;
            }
            else if y1 == y2 {
                y1 += 0.5;
                y2 += 0.5;
                x2 += 1.;
            }
            draw_line(x1 as f32, y1 as f32, x2 as f32, y2 as f32, 1., (&color_palette.borrow())[map_pallete_color(color)]);
            
        };

        let rect = |x1: i32, y1: i32, w: u32, h: u32, color: &DrawColor| {
            draw_rectangle_lines(x1 as f32, y1 as f32, w as f32, h as f32, 1., (&color_palette.borrow())[map_pallete_color(color)])
        };

        let text_str = |t: &str, x: i32, y: i32, color: &DrawColor| {
            draw_text_ex(
                t,
                x as f32,
                (y + FONT_HEIGHT as i32) as f32,
                TextParams {
                    font_size: FONT_HEIGHT,
                    font: Some(&font),
                    font_scale: 1.,
                    color: (&color_palette.borrow())[map_pallete_color(color)],
                    ..Default::default()
                },
            );
        };

        let mut switch_palette = |pallette: &Pallette| {
            fn map_color(color_as_u32: u32) -> Color {
                color_u8!(
                    ((color_as_u32 & 0x00ff0000) >> 16) & 0xff,
                    ((color_as_u32 & 0x0000ff00) >> 8) & 0xff,
                    color_as_u32 & 0xff,
                    0xff
                )
            }

            bg_color = map_color(pallette.background);

            let new_colormap = build_colormap(
                KITTY_SS_COLORS,
                [
                    map_color(pallette.main_kitty),
                    map_color(pallette.pigs_lizards),
                    map_color(pallette.foreground),
                    bg_color,
                    BLANK
                ]
            );

            let cp = &mut color_palette.borrow_mut();

            let len = cp.len();

            for i in 0..cp.len() - 1 {
                cp[i] = new_colormap[&KITTY_SS_COLORS[i]];
            }
            cp[len - 1] = BLANK;

            kitty_ss_texture.update(&recolor_spritesheet(&original_image, new_colormap));
        };

        let mut btns_pressed_this_frame = [0; 4];
        let mut gamepads = [0; 4];
        


        let mut keymap = HashMap::with_capacity(6);
        keymap.insert(KeyCode::X, BUTTON_1);
        keymap.insert(KeyCode::Z, BUTTON_2);
        keymap.insert(KeyCode::Space, BUTTON_1);

        keymap.insert(KeyCode::Left, BUTTON_LEFT);
        keymap.insert(KeyCode::Right, BUTTON_RIGHT);
        keymap.insert(KeyCode::Up, BUTTON_UP);
        keymap.insert(KeyCode::Down, BUTTON_DOWN);

        for (keycode, input) in &keymap {
            if is_key_pressed(*keycode) {
                btns_pressed_this_frame[0] |= input; 
            }
        }
        for (keycode, input) in &keymap {
            if is_key_down(*keycode) {
                gamepads[0] |= input; 
            }
        }  

        kittygame_update(&blit_sub, &line, &rect, &text_str, &mut switch_palette, internal_width as u32, internal_height as u32, &btns_pressed_this_frame, &gamepads);
        next_frame().await
    }
}
