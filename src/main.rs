use macroquad::prelude::*;

/// Clone an image, replacing the colors with some colormap.
// fn recolor_spritesheet() {}

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

    let kitty_bg_texture: Texture2D = load_texture("kittygame.png").await.unwrap();
    let kitty_ss_texture: Texture2D = load_texture("kitty-ss.png").await.unwrap();

    // let font = load_ttf_font("SRAFreePixelFontPack/PixelSmall.ttf")

    let font = load_ttf_font("Pixeloid_Font_0_5/TrueType (.ttf)/PixeloidSans.ttf")
        .await
        .unwrap();

    kitty_bg_texture.set_filter(FilterMode::Nearest);
    kitty_ss_texture.set_filter(FilterMode::Nearest);

    // we only want to create a new texture (and image) when necessary, because
    // it's expensive. So track when the screen changes dimensions.
    let mut last_sh = screen_height();
    let mut last_sw = screen_width();

    let mut fps = 0;
    let mut i = 0;
    loop {
        i += 1;
        if i % 15 == 0 {
            fps = get_fps();
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

        clear_background(LIGHTGRAY);
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

        draw_texture(&kitty_bg_texture, 0., 0., WHITE);
        for _ in 0..10000 {
            draw_texture_ex(
                &kitty_ss_texture,
                10.,
                10.,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(16., 56., 16., 8.)),
                    ..Default::default()
                },
            );
        }

        draw_text_ex(
            &format!["CanyonTurtle: {}", fps],
            10.,
            10.,
            TextParams {
                font_size: 9,
                font: Some(&font),
                font_scale: 1.,
                ..Default::default()
            },
        );
        next_frame().await
    }
}
