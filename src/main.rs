
use macroquad::prelude::*;

/// Clone an image, replacing the colors with some colormap.
fn recolor_spritesheet() {}

/// Takes dest image, source image, source x/y, dest x/y, width/height, spritesheet stride.
/// Destination coords are signed because this can attempt to draw things offscreen to the sides.
fn blit_sub(
    spritesheet: &Image,
    destination_image: &mut Image,
    src_x: u32,
    src_y: u32,
    dest_x: i32,
    dest_y: i32,
    width: usize,
    height: usize,

) {
    // bounds checks. this function will never fail, so it must smartly
    // clip off anything that won't be rendered.
    let actual_draw_width = (width as i32)
        .min(spritesheet.width() as i32 - src_x as i32)
        .min(destination_image.width() as i32 - dest_x as i32);
    if actual_draw_width <= 0 {
        return;
    }
    let actual_draw_height = (height as i32)
        .min(spritesheet.height() as i32 - src_y as i32)
        .min(destination_image.height() as i32 - dest_y as i32);
    if actual_draw_height <= 0 {
        return;
    }

    // By the time these checks are done, we know:
    // - if we're still drawing, we have a valid source and destination position,
    // and a valid actual draw width and height that keeps the other parts in bounds.

    let dest_im_stride = destination_image.width();
    let spritesheet_stride = spritesheet.width();

    let dest_image_data = destination_image.get_image_data_mut();
    let spritesheet_data = spritesheet.get_image_data();
    // for each row, do (now non-zero) draw.
    for i in 0..actual_draw_height as usize {
        let dest_row = dest_y as usize + i;
        let dest_start_loc = dest_im_stride * dest_row + dest_x as usize;
        let src_row = src_y as usize + i;
        let src_start_loc = spritesheet_stride * src_row + src_x as usize;
        for (dest_px, src_px) in dest_image_data[dest_start_loc..(dest_start_loc+actual_draw_width as usize) as usize].iter_mut().zip(spritesheet_data[src_start_loc..src_start_loc+actual_draw_width as usize].iter()) {
            *dest_px = *src_px;
        }
    }
}


fn window_conf() -> Conf {
    Conf {
        window_title: "Window name".to_owned(),
        window_width: 1600,
        window_height: 900,
        fullscreen: false,
        window_resizable: true,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    set_pc_assets_folder("assets");
    const MAX_SCREEN_DIM: f32 = 400.;

    
    
    // let mut im = texture.get_texture_data();
    let mut game_framebuffer = Image{
        bytes: vec![0; MAX_SCREEN_DIM.powi(2) as usize * 4],
        width: MAX_SCREEN_DIM as u16,
        height: MAX_SCREEN_DIM as u16,
    };
    let texture: Texture2D = Texture2D::from_image(&game_framebuffer.clone());

    let kitty_texture: Texture2D = load_texture("kittygame.png").await.unwrap();
    let kitty_image = kitty_texture.get_texture_data();
    let kitty_image_copy = kitty_image.clone();
    let kitty_raw_image_bytes = kitty_image.get_image_data();

    
    texture.set_filter(FilterMode::Nearest);
    
    let mut i = 0;

    // we only want to create a new texture (and image) when necessary, because
    // it's expensive. So track when the screen changes dimensions.
    let mut last_sh = screen_height();
    let mut last_sw = screen_width();

    loop {

        // we want the dimensions of the screen to be:
        // minimum internal dimension is 160
        // maximum dimension is as large as will fill the screen

        let min_internal_dim = 160;

        let sh = screen_height();
        let sw = screen_width();

        if last_sh != sh || last_sw != sw{
            last_sh = sh;
            last_sw = sw;
            // create new texture
        }

        enum Dim {
            Width,
            Height
        }

        

        let smaller_side = match sh <= sw {
            true => {
                Dim::Height
            },
            _ => Dim::Width
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

        clear_background(LIGHTGRAY);
        draw_texture_ex(&texture, 0., 0., WHITE, DrawTextureParams{
            source: Some(Rect { x:0.0, y: 0.0, w: internal_width as f32, h: internal_height as f32 }),
            dest_size: Some(vec2(sw, sh)),
            ..Default::default()
        });
        for (j, c) in game_framebuffer.get_image_data_mut().iter_mut().enumerate() {
            *c = kitty_raw_image_bytes[(j + i) % (MAX_SCREEN_DIM.powi(2) as usize)];
        }
        for _ in 0..100{
            blit_sub(
                &kitty_image_copy, 
                &mut game_framebuffer,
                70,
                80,
                10,
                10,
                30,
                20
            );
        }
        
        texture.update(&game_framebuffer);
        i += 1;
        draw_text_ex(&format!["{} fps", get_fps()], 30., 30., TextParams{
            color: color_u8![255, 0, 0, 255],
            ..Default::default()
        });
        next_frame().await
    }
}