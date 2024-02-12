use std::{cell::RefCell, collections::HashMap};

use macroquad::prelude::*;

use kittygame::{kittygame_update, multiplatform_defs::{BlitSubFlags, DrawColor, Pallette, Spritesheet, BUTTON_1, BUTTON_2, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP}};


const ORIGINAL_KITTY_SS_COLORS: [[u8; 4]; 5] = [
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

const ORIGINAL_TITLE_COLORS: [[u8; 4]; 5] = [
    [0xF9, 0xDF, 0xD1, 0xff], // main letter color
    [0xEB, 0x9F, 0x9E, 0xff], // letter backing color
    [0x12, 0x00, 0x00, 0x00], // transparent (unused)
    [0x34, 0x00, 0x00, 0x00], // transparent (unused)
    [0x00, 0x00, 0x00, 0x00], // transparent
];

const ORIGINAL_GAMEPAD_COLORS: [[u8; 4]; 5] = [
    [0xac, 0x32, 0x32, 0xff], // main letter color
    [0x22, 0x20, 0x34, 0xff], // letter backing color (maps to transparent)
    [0x12, 0x00, 0x00, 0x00], // (unused)
    [0x34, 0x00, 0x00, 0x00], // (unused)
    [0x00, 0x00, 0x00, 0x00], // (unused)
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

    /// the first color is the foreground color, 
    /// and the others are transparent.
    fn construct_gamepad_colors(colors: [Color; 5]) -> ([Color; 5], [Color; 5]) {
        (
            [
                colors[0],
                BLANK,
                BLANK,
                BLANK,
                BLANK
            ],
            [
                colors[1],
                BLANK,
                BLANK,
                BLANK,
                BLANK
            ],
        )
    }

    enum InputMode {
        KeyboardDetected,
        Touchpad
    }

    let mut current_input_mode = InputMode::Touchpad;

    let color_palette = RefCell::new(DEFAULT_COLOR_PALLETTE);

    set_pc_assets_folder("assets");

    const MAX_SCREEN_DIM: f32 = 400.;

    // let kitty_bg_texture: Texture2D = load_texture("kittygame.png").await.unwrap();
    let kitty_ss_texture: Texture2D = load_texture("kitty-ss.png").await.unwrap();
    let kitty_title_texture: Texture2D = load_texture("kitty_title.png").await.unwrap();
    let gamepad_texture: Texture2D = load_texture("gamepad.png").await.unwrap();

    // let font = load_ttf_font("SRAFreePixelFontPack/PixelSmall.ttf")

    // const FONT_HEIGHT: u16 = 9;
    // let mut font = load_ttf_font("Pixeloid_Font_0_5/TrueType (.ttf)/PixeloidSans.ttf")

    const FONT_HEIGHT: u16 = 8;

    let mut font = load_ttf_font("PressStart2P-Regular.ttf")
        .await
        .unwrap();

    font.set_filter(FilterMode::Nearest);

    let original_image = kitty_ss_texture.get_texture_data();

    let original_title_image = kitty_title_texture.get_texture_data();

    let original_gamepad_image = gamepad_texture.get_texture_data();

    let pressed_gamepad_image = gamepad_texture.get_texture_data();

    let pressed_gamepad_texture = Texture2D::from_image(&pressed_gamepad_image);

    let recolor_textures_from_pallette = |color_pallette: [Color; 5]| {
        let recolored_ss = recolor_spritesheet(&original_image, build_colormap(ORIGINAL_KITTY_SS_COLORS, color_pallette.clone()));
        kitty_ss_texture.update(&recolored_ss);
        
        let recolored_title = recolor_spritesheet(&original_title_image, build_colormap(ORIGINAL_TITLE_COLORS, color_pallette.clone()));
        kitty_title_texture.update(&recolored_title);
    
        let (gamepad_color_pallette, pressed_color_pallette) = construct_gamepad_colors(color_pallette.clone());

        let recolored_gamepad = recolor_spritesheet(&original_gamepad_image, build_colormap(ORIGINAL_GAMEPAD_COLORS, gamepad_color_pallette.clone()));
        gamepad_texture.update(&recolored_gamepad);
        

        let recored_pressed_gamepad = recolor_spritesheet(&pressed_gamepad_image, build_colormap(ORIGINAL_GAMEPAD_COLORS, pressed_color_pallette));
        pressed_gamepad_texture.update(&recored_pressed_gamepad);
    };

    recolor_textures_from_pallette((*color_palette.borrow()).clone());



    // kitty_bg_texture.set_filter(FilterMode::Nearest);
    kitty_ss_texture.set_filter(FilterMode::Nearest);
    kitty_title_texture.set_filter(FilterMode::Nearest);
    gamepad_texture.set_filter(FilterMode::Nearest);


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

        let blit_sub = &mut |spritesheet: Spritesheet, x: i32, y: i32, w: u32, h: u32, src_x: u32, src_y: u32, flags: BlitSubFlags| {
            const CLIP_OFF_EPS: f32 = -0.1;
            draw_texture_ex(
                match spritesheet {
                    Spritesheet::Main => &kitty_ss_texture,
                    Spritesheet::Title => &kitty_title_texture,
                },
                x as f32 + CLIP_OFF_EPS,
                y as f32 + CLIP_OFF_EPS,
                WHITE,
                DrawTextureParams{
                    source: Some(Rect{
                        x: src_x as f32 + CLIP_OFF_EPS,
                        y: src_y as f32 + CLIP_OFF_EPS,
                        w: w as f32 - 2. * CLIP_OFF_EPS,
                        h: h as f32 - 2. * CLIP_OFF_EPS
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

        let line = &mut |x1i: i32, y1i: i32, x2i: i32, y2i: i32, color: &DrawColor| {
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

        let rect = &mut |x1: i32, y1: i32, w: u32, h: u32, color: &DrawColor| {
            draw_rectangle_lines(x1 as f32, y1 as f32, w as f32, h as f32, 1., (&color_palette.borrow())[map_pallete_color(color)])
        };

        let text_str = &mut |t: &str, x: i32, y: i32, color: &DrawColor| {
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

        let switch_palette = &mut |pallette: &Pallette| {
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
                ORIGINAL_KITTY_SS_COLORS,
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
                cp[i] = new_colormap[&ORIGINAL_KITTY_SS_COLORS[i]];
            }
            cp[len - 1] = BLANK;

            // kitty_ss_texture.update(&recolor_spritesheet(&original_image, new_colormap));

            recolor_textures_from_pallette(cp.clone());
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
                current_input_mode = InputMode::KeyboardDetected;
                btns_pressed_this_frame[0] |= input; 
            }
        }
        for (keycode, input) in &keymap {
            if is_key_down(*keycode) {
                gamepads[0] |= input; 
            }
        }  

        /// location of arrow on the spritesheet.
        const ARROW_SPRITE_RECT: Rect = Rect{
            x: 32.,y: 0.,w: 35.,h: 32.
        };

        /// Location of button on the spritesheet.
        const BUTTON_SPRITE_RECT: Rect = Rect{
            x: 0., y: 0., w: 29., h: 29.
        };

        const GAMEPAD_OFFSET_FROM_BOTTOM: f32  = 60.;

        let left_arrow_pos: Vec2 = Vec2{x: 10., y: internal_height as f32 - GAMEPAD_OFFSET_FROM_BOTTOM};
        let right_arrow_pos: Vec2 = Vec2{x: 50., y: internal_height as f32 - GAMEPAD_OFFSET_FROM_BOTTOM};
        let x_button_pos: Vec2 = Vec2{x: internal_width as f32 - 70., y: internal_height as f32 - GAMEPAD_OFFSET_FROM_BOTTOM + 10.};
        let z_button_pos: Vec2 = Vec2{x: internal_width as f32 - 40., y: internal_height as f32 - GAMEPAD_OFFSET_FROM_BOTTOM - 10.};

        let touch_zones: [Rect; 4] = [
            ARROW_SPRITE_RECT.offset(left_arrow_pos - Vec2{x: ARROW_SPRITE_RECT.x, y: ARROW_SPRITE_RECT.y}),
            ARROW_SPRITE_RECT.offset(right_arrow_pos - Vec2{x: ARROW_SPRITE_RECT.x, y: ARROW_SPRITE_RECT.y}),
            BUTTON_SPRITE_RECT.offset(x_button_pos - Vec2{x: BUTTON_SPRITE_RECT.x, y: BUTTON_SPRITE_RECT.y}),
            BUTTON_SPRITE_RECT.offset(z_button_pos - Vec2{x: BUTTON_SPRITE_RECT.x, y: BUTTON_SPRITE_RECT.y}),
        ];
        let touch_buttons: [u8; 4] = [BUTTON_LEFT, BUTTON_RIGHT, BUTTON_1, BUTTON_2];



        for touch in touches_local() {
            let position_internal = Vec2{
                x: ((touch.position.x * 0.5) + 0.5) * internal_width as f32,
                y: ((touch.position.y * 0.5) + 0.5) * internal_height as f32,
            };
            match touch.phase {
                TouchPhase::Started => {
                    for i in 0..touch_zones.len() {
                        let touch_zone = touch_zones[i];
                        let touch_button = touch_buttons[i];
                        
                        if touch_zone.contains(position_internal) {
                            btns_pressed_this_frame[0] |= touch_button;
                            gamepads[0] |= touch_button;
                            trace!("hit");
                        }
                    }
                },
                TouchPhase::Ended => {

                },
                _ => {
                    for i in 0..touch_zones.len() {
                        let touch_zone = touch_zones[i];
                        let touch_button = touch_buttons[i];
                        if touch_zone.contains(position_internal) {
                            gamepads[0] |= touch_button;
                        }
                    }
                },
            }
        }

        kittygame_update(blit_sub, line, rect, text_str, switch_palette, internal_width as u32, internal_height as u32, &btns_pressed_this_frame, &gamepads);

        match current_input_mode {
            InputMode::KeyboardDetected => {},
            InputMode::Touchpad => {

                let left_texture = match &gamepads[0] & BUTTON_LEFT != 0 {
                    true => &pressed_gamepad_texture,
                    false => &gamepad_texture
                };
                let right_texture = match &gamepads[0] & BUTTON_RIGHT != 0 {
                    true => &pressed_gamepad_texture,
                    false => &gamepad_texture
                };
                let x_texture = match &gamepads[0] & BUTTON_1 != 0 {
                    true => &pressed_gamepad_texture,
                    false => &gamepad_texture
                };
                let z_texture = match &gamepads[0] & BUTTON_2 != 0 {
                    true => &pressed_gamepad_texture,
                    false => &gamepad_texture
                };


                // left arrow
                draw_texture_ex(left_texture, left_arrow_pos.x, left_arrow_pos.y, WHITE, DrawTextureParams{
                    source: Some(ARROW_SPRITE_RECT),
                    flip_x: true,
                    ..Default::default()
                });
                // right arrow
                draw_texture_ex(right_texture, right_arrow_pos.x, right_arrow_pos.y, WHITE, DrawTextureParams{
                    source: Some(ARROW_SPRITE_RECT),
                    ..Default::default()
                });

                // x
                draw_texture_ex(x_texture, x_button_pos.x, x_button_pos.y, WHITE, DrawTextureParams{
                    source: Some(BUTTON_SPRITE_RECT),
                    ..Default::default()
                });

                // z
                draw_texture_ex(z_texture, z_button_pos.x, z_button_pos.y, WHITE, DrawTextureParams{
                    source: Some(BUTTON_SPRITE_RECT),
                    ..Default::default()
                });
            },
        }
        next_frame().await
    }
}
