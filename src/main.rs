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
    let texture: Texture2D = load_texture("src/kittygame.png").await.unwrap();
    let mut im = texture.get_texture_data();
    let imc = im.clone();
    let imcp = imc.get_image_data();
    texture.set_filter(FilterMode::Nearest);
    
    let mut i = 0;
    loop {
        let sl = screen_height().min(screen_width());
        clear_background(LIGHTGRAY);
        draw_texture_ex(&texture, 0., 0., WHITE, DrawTextureParams{
            dest_size: Some(vec2(sl, sl)),
            ..Default::default()
        });
        for (j, c) in im.get_image_data_mut().iter_mut().enumerate() {
            *c = imcp[(j + i) % (160*160)];
        }
        texture.update(&im);
        i += 2;
        next_frame().await
    }
}