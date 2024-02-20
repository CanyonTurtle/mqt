//! Shows how to create graphics that snap to the pixel grid by rendering to a texture in 2D


use bevy::{
    prelude::*, render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages
        },
        view::RenderLayers,
    }, sprite::Anchor, window::WindowResized
};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use kittygame::{kittygame_update, multiplatform_defs::{BlitSubFlags, LineFunc, RectFunc, SwitchPalletteFunc, TextStrFunc}};
use kittygame::multiplatform_defs;

/// In-game resolution width.
const RES_WIDTH: u32 = 320;

/// In-game resolution height.
const RES_HEIGHT: u32 = 160;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

#[derive(Resource)]
pub struct SpritesThisFrame {
    pub sprites: Vec<Entity>
}

#[derive(Resource)]
pub struct PreviousFrameInput([u8; 4]);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(bevy_framepace::FramepacePlugin)
        .insert_resource(Msaa::Off)
        .insert_resource(SpritesThisFrame{sprites: vec![]})
        .insert_resource(PreviousFrameInput{0: [0; 4]})
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (fit_canvas, kittygame_update_bevy))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())

        .run();
}

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
struct OuterCamera;

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // this camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            ..default()
        },
        InGameCamera,
        PIXEL_PERFECT_LAYERS,
    ));

    // spawn the canvas
    commands.spawn((
        SpriteBundle {
            texture: image_handle,
            ..default()
        },
        Canvas,
        HIGH_RES_LAYERS,
    ));

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((Camera2dBundle::default(), OuterCamera, HIGH_RES_LAYERS));

    
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        let mut projection = projections.single_mut();
        projection.scale = 1. / h_scale.min(v_scale);
    }
}


// use bevy::prelude::*;
// use bevy_pixel_buffer::prelude::*;

// fn main() {
//     let size = PixelBufferSize {
//         size: UVec2::new(320, 160),       // amount of pixels
//         pixel_size: UVec2::new(16, 16), // size of each pixel in the screen
//     };

//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_plugins(PixelBufferPlugin)  // Add this plugin
//         .add_systems(Startup, pixel_buffer_setup(size)) // Setup system
//         .add_systems(Update, update)
//         .run()
// }

// fn update(mut pb: QueryPixelBuffer) {
//     // Set each pixel to a random color
//     pb.frame().per_pixel(|_, _| Pixel::random());
// }

// implementation plan:
// provide update with blit sub
// spawn a sprite for every call to blit_sub
// don't do line.

/// Rotates entities to demonstrate grid snapping.
fn kittygame_update_bevy(mut commands: Commands, asset_server: Res<AssetServer>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, mut sprites_this_frame: ResMut<SpritesThisFrame>, keyboard_input: Res<ButtonInput<KeyCode>>, mut previous_input: ResMut<PreviousFrameInput>) {
    // commands.spawn((
    //     SpriteBundle {
    //         texture: asset_server.load("bevy_pixel_dark.png"),
    //         transform: Transform::from_xyz(-40., 20., 2.),
    //         ..default()
    //     },
    //     Rotate,
    //     PIXEL_PERFECT_LAYERS,
    // ));

    for e in &mut sprites_this_frame.sprites {
        commands.entity(*e).despawn();
    }
    sprites_this_frame.sprites.clear();

    let mut zc = 1.0;

    let blit_sub = &mut |spritesheet: multiplatform_defs::Spritesheet, x: i32, y: i32, w: u32, h: u32, src_x: u32, src_y: u32, flags: BlitSubFlags| {
        let ss_str = match spritesheet {
            multiplatform_defs::Spritesheet::Main => "../../assets/kitty-ss.png",
            multiplatform_defs::Spritesheet::Title => "../../assets/kitty_title.png",
        };
        let texture_handle = asset_server.load(ss_str);
        let mut texture_atlas = TextureAtlasLayout::new_empty(Vec2{x: 192., y: 64.});
        texture_atlas.add_texture(Rect::new(src_x as f32, src_y as f32, src_x as f32 + w as f32, src_y as f32 + h as f32));
        let texture_atlas_layout = texture_atlas_layouts.add(texture_atlas);

        let mut st = Transform::from_xyz(x as f32 - (RES_WIDTH / 2) as f32, -y as f32 + (RES_HEIGHT / 2) as f32, zc);
        zc += 1.0;

        if flags.flip_x {
            st.scale *= Vec3{x: -1., y: 1., z: 1.};
        }
        if flags.flip_y {
            st.scale *= Vec3{x: 1., y: -1., z: 1.};
        }
        let anchor;
        if flags.flip_x && !flags.flip_y {
            anchor = Anchor::TopRight;
        } else if !flags.flip_x && flags.flip_y {
            anchor = Anchor::BottomLeft
        } else if flags.flip_x {
            anchor = Anchor::BottomRight;
        } else {
            anchor = Anchor::TopLeft;
        }

        let handle = commands.spawn((
            SpriteSheetBundle {
                sprite: Sprite{
                    anchor,
                    ..Default::default()
                },
                texture: texture_handle,
                atlas: TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                },
                transform: st,
                ..Default::default()
            },  
            PIXEL_PERFECT_LAYERS
        )).id();
        sprites_this_frame.sprites.push(handle);
    };
    let line: &mut LineFunc = &mut |_, _, _, _, _| {

    };

    let rect: &mut RectFunc = &mut |_, _, _, _, _| {

    };

    let text_str: &mut TextStrFunc = &mut |_, _, _, _| {};

    let switch_palette: &mut SwitchPalletteFunc = &mut |_| {};

    let mut btns_pressed_this_frame = [0; 4];
    let mut gamepads = btns_pressed_this_frame.clone();

    const INPUT_MAPPING: &[(KeyCode, u8)] = &[
        (KeyCode::ArrowLeft, kittygame::multiplatform_defs::BUTTON_LEFT),
        (KeyCode::ArrowRight, kittygame::multiplatform_defs::BUTTON_RIGHT),
        (KeyCode::ArrowUp, kittygame::multiplatform_defs::BUTTON_UP),
        (KeyCode::ArrowDown, kittygame::multiplatform_defs::BUTTON_DOWN),
        (KeyCode::KeyZ, kittygame::multiplatform_defs::BUTTON_1),
        (KeyCode::Space, kittygame::multiplatform_defs::BUTTON_1),
        (KeyCode::KeyX, kittygame::multiplatform_defs::BUTTON_2),
    ];

    for (keycode, input_bit) in INPUT_MAPPING.into_iter() {
        if keyboard_input.pressed(*keycode) {
            gamepads[0] |= input_bit;
            if previous_input.0[0] & input_bit == 0 {
                btns_pressed_this_frame[0] |= input_bit;
            }
        } 
    }

    previous_input.0 = btns_pressed_this_frame;

 


    kittygame_update(blit_sub, line, rect, text_str, switch_palette, RES_WIDTH as u32, RES_HEIGHT as u32, &btns_pressed_this_frame, &gamepads);

}