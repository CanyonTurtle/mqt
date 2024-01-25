use macroquad::prelude::*;

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

    const MAX_SCREEN_DIM: f32 = 400.;

    let png_texture: Texture2D = load_texture("src/kittygame.png").await.unwrap();
    
    // let mut im = texture.get_texture_data();
    let mut im = Image{
        bytes: vec![0; MAX_SCREEN_DIM.powi(2) as usize * 4],
        width: MAX_SCREEN_DIM as u16,
        height: MAX_SCREEN_DIM as u16,
    };
    let texture: Texture2D = Texture2D::from_image(&im.clone());
    let imc = png_texture.get_texture_data();
    let imcp = imc.get_image_data();
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
        for (j, c) in im.get_image_data_mut().iter_mut().enumerate() {
            *c = imcp[(j + i) % (MAX_SCREEN_DIM.powi(2) as usize)];
        }
        texture.update(&im);
        i += 1;
        next_frame().await
    }
}