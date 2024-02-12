//! Kitty game!
//!
//! [`kittygame`]: https://canyonturtle.github.io/kittygame/

/// This is essentially the entrypoint of the game, providing the update() loop. 
/// This has all the drawing code and lots of update logic.

#[cfg(feature = "buddy-alloc")]
mod alloc;

// mod kitty_ss;
pub mod spritesheet;

pub mod multiplatform_defs;

//#[cfg(feature = "wasm-4")]
// mod wasm4;

use game::{
    camera::Camera,
    collision::{check_entity_collisions, update_pos},
    entities::{Character, MovingEntity, KittyStates, WarpAbility, WarpState},
    game_constants::{
        MAX_N_NPCS, TILE_HEIGHT_PX, TILE_WIDTH_PX, X_LEFT_BOUND, X_RIGHT_BOUND, Y_LOWER_BOUND,
        Y_UPPER_BOUND,
    },
    game_state::GameState,
    menus::GameMode,
    music::{play_bgm, SONGS}, game_map::MAP_TILESETS, cloud::Cloud,
};
use multiplatform_defs::{BlitSubFlags, BlitSubFunc, DrawColor, LineFunc, RectFunc, Spritesheet, SwitchPalletteFunc, TextStrFunc};

// use title_ss::{OUTPUT_ONLINEPNGTOOLS_WIDTH, OUTPUT_ONLINEPNGTOOLS_HEIGHT};

const OUTPUT_ONLINEPNGTOOLS_WIDTH: u32 = 152;
const OUTPUT_ONLINEPNGTOOLS_HEIGHT: u32 = 50;

mod game;
// mod title_ss;

use crate::{game::{
        collision::{get_bound_of_character, AbsoluteBoundingBox},
        entities::OptionallyEnabledPlayer,
        menus::{Modal, NormalPlayModes, MenuTypes, SelectSetup, SelectMenuFocuses}, game_constants::{COUNTDOWN_TIMER_START, FINAL_LEVEL, INCR_VERSION, LEVELS_PER_MOOD, MAJOR_VERSION, MINOR_VERSION, START_DIFFICULTY_LEVEL}, popup_text::{PopTextRingbuffer, PopupIcon}, rng::{GameRng, Rng}, game_state::RunType,}, multiplatform_defs::{Pallette, BUTTON_1, BUTTON_2, BUTTON_LEFT, BUTTON_RIGHT}};

/// draw the tiles in the map, relative to the camera.
fn drawmap(game_state: &GameState, blit_sub: &mut BlitSubFunc, sw: u32, sh: u32) {
    let map = &game_state.map;
    let camera = &game_state.camera;

    let tileset = &MAP_TILESETS[game_state.tileset_idx];

    for chunk in &map.chunks {
        for row in 0..chunk.bound.height {
            for col in 0..chunk.bound.width {
                let map_tile_i = chunk.get_tile(col as usize, row as usize);
                match map_tile_i {
                    0 => {}
                    tile_idx => {
                        let tile_i: usize = tileset[tile_idx as usize] as usize; // *tile_idx as usize;
                        if tile_i == 0 {
                            continue
                        }                                   // trace(format!("Tile {tile_i}"));
                        let chunk_x_offset: i32 = (TILE_WIDTH_PX) as i32 * chunk.bound.x;
                        let chunk_y_offset: i32 = (TILE_HEIGHT_PX) as i32 * chunk.bound.y;
                        let x_loc = (chunk_x_offset + col as i32 * TILE_HEIGHT_PX as i32)
                            - camera.current_viewing_x_offset as i32;
                        let y_loc = (chunk_y_offset + row as i32 * TILE_WIDTH_PX as i32)
                            - camera.current_viewing_y_offset as i32;

                        if x_loc >= 0 && x_loc < sw as i32 && y_loc > 0 && y_loc < sh as i32 {
                            blit_sub(
                                Spritesheet::Main,
                                x_loc,
                                y_loc,
                                game_state.background_tiles[tile_i].frames[0].width as u32,
                                game_state.background_tiles[tile_i].frames[0].height as u32,
                                game_state.background_tiles[tile_i].frames[0].start_x as u32,
                                game_state.background_tiles[tile_i].frames[0].start_y as u32,
                                BlitSubFlags{flip_x: false, flip_y: false},
                            );
                        }
                    }
                }
            }
        }
    }
}

static mut GAME_STATE_HOLDER: Option<GameState> = None;

/// Draw a character on-screen, relative to the camera.
fn drawcharacter(
    camera: &Camera,
    character: MovingEntity,
    blit_sub: &mut BlitSubFunc
) {
    let the_char: &mut Character;

    match character {
        MovingEntity::OptionalPlayer(optionally_enabled_player) => {
            match optionally_enabled_player {
                OptionallyEnabledPlayer::Enabled(p) => {
                    the_char = &mut p.character;
                }
                OptionallyEnabledPlayer::Disabled => return,
            }
        }
        MovingEntity::NPC(npc) => {
            the_char = npc;
        }
    }

    let i = the_char.current_sprite_i as usize;
    blit_sub(
        Spritesheet::Main,
        (the_char.x_pos - camera.current_viewing_x_offset) as i32,
        (the_char.y_pos - camera.current_viewing_y_offset) as i32,
        the_char.sprite.frames[i].width as u32,
        the_char.sprite.frames[i].height as u32,
        the_char.sprite.frames[i].start_x as u32,
        the_char.sprite.frames[i].start_y as u32,
        BlitSubFlags{
            flip_x: !the_char.is_facing_right,
            flip_y: match the_char.state {
                KittyStates::OnCeiling(_) => true,
                _ => false
            }
        }
    );
}

static mut NPC_INPUTS: [u8; MAX_N_NPCS] = [0; MAX_N_NPCS];



// static mut PREVIOUS_GAMEPAD: [u8; 4] = [0, 0, 0, 0];

/// get joystick inputs from this and last frame.
// fn get_inputs_this_frame() -> [[u8; 4]; 2] {
//     let gamepads: [u8; 4] = unsafe { [*GAMEPAD1, *GAMEPAD2, *GAMEPAD3, *GAMEPAD4] };
//     let mut btns_pressed_this_frame: [u8; 4] = [0; 4];

//     for i in 0..gamepads.len() {
//         let gamepad = gamepads[i];
//         let previous = unsafe { PREVIOUS_GAMEPAD[i] };
//         let pressed_this_frame = gamepad & (gamepad ^ previous);
//         btns_pressed_this_frame[i] = pressed_this_frame;
//     }
//     unsafe { PREVIOUS_GAMEPAD.copy_from_slice(&gamepads[0..4]) };
//     [btns_pressed_this_frame, gamepads]
// }

const TOP_UI_TEXT_Y: i32 = 2;
const BOTTOM_UI_TEXT_Y_OFFSET: i32 = -8; // 160 - 8 - 2;

/// DRAW BLURRED BACKGROUND BEHIND SCORE AND TIME TEXTS IN-GAME
fn draw_modal_bg(pf: &AbsoluteBoundingBox<f32, f32>, style: u8, color: &DrawColor, line: &mut LineFunc, rect: &mut RectFunc) {
    // unsafe { *DRAW_COLORS = color }
    let p: AbsoluteBoundingBox<i32, u32> = AbsoluteBoundingBox {
        x: pf.x as i32,
        y: pf.y as i32,
        width: pf.width as u32,
        height: pf.height as u32,
    };

    // unsafe {
    //     *DRAW_COLORS = 0x0001;
    // }

    match style {
        1 => {
            rect(p.x, p.y, p.width as u32, p.height as u32, &DrawColor::Background);
        }
        _ => {}
    }

    // unsafe {
    //     *DRAW_COLORS = match style {
    //         0 => 0x0001,
    //         _ => 0x0002,
    //     }
    // };

    // fill
    for i in p.x..=p.x + p.width as i32 {
        for j in p.y..=p.y + p.height as i32 {
            let cond = match style {
                1 => false,
                _ => (i + j) % 3 != 0,
            };
            if cond {
                line(i, j, i, j, &DrawColor::Background)
            }
        }
    }

    // unsafe { *DRAW_COLORS = color }
    // borders

    match style {
        1 => {
            line(p.x, p.y, p.x + p.width as i32, p.y, &color);
            line(p.x, p.y, p.x, p.y + p.height as i32, &color);
            line(
                p.x,
                p.y + p.height as i32,
                p.x + p.width as i32,
                p.y + p.height as i32,
                &color
            );
            line(
                p.x + p.width as i32,
                p.y,
                p.x + p.width as i32,
                p.y + p.height as i32,
                &color
            );
        }
        _ => {}
    }
    
}

/// Draw text with a soft background under
fn layertext(t: &str, x: i32, y: i32, text_str: &mut TextStrFunc) {
    // unsafe { *DRAW_COLORS = 0x0001 }
    text_str(t, x + 1, y, &DrawColor::Background);
    text_str(t, x, y + 1, &DrawColor::Background);
    text_str(t, x + 1, y + 1, &DrawColor::Background);
    // unsafe { *DRAW_COLORS = 0x0002 }

    text_str(t, x, y, &DrawColor::Foreground);
}

const TIMER_INTERACTIVE_START: u32 = 100;
const TITLE_Y: i32 = 15;

fn render_title(game_state: &GameState, x: i32, y: i32, blit_sub: &mut BlitSubFunc) {
    // RENDER THE TITLE
    // unsafe { *DRAW_COLORS = 0x0034 }
    let title_x: i32 = x;
    let title_y_osc = match game_state.song_timer {
        0..=TIMER_INTERACTIVE_START => {
            0
        }
        _ => {
            (5f32 * num::Float::sin((game_state.song_timer - TIMER_INTERACTIVE_START) as f32 * 0.05f32)) as i32
        }
    };
    for row in 0..OUTPUT_ONLINEPNGTOOLS_HEIGHT as i32 {
        blit_sub(Spritesheet::Title, title_x + (3000000f32 * (1f32 / (1f32 + num::Float::powf(game_state.song_timer as f32, 3f32))) * num::Float::sin((game_state.song_timer as f32 + row as f32 * 4f32) * 0.1f32)) as i32, y + title_y_osc + row, OUTPUT_ONLINEPNGTOOLS_WIDTH, 1, 0, row as u32, BlitSubFlags { flip_x: false, flip_y: false })
    }
    // unsafe {
    //     *PALETTE = spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx];
    // }
    // unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
}


/// Main loop that runs every frame. Progress the game state and render.
#[no_mangle]
pub fn kittygame_update(blit_sub: &mut BlitSubFunc, line: &mut LineFunc, rect: &mut RectFunc, text_str: &mut TextStrFunc, set_palette: &mut SwitchPalletteFunc, sw: u32, sh: u32, btns_pressed_this_frame: &[u8; 4], gamepads: &[u8; 4]) {
    
    let (center_x, center_y) = (sw as f32 / 2., sh as f32 / 2.);
    
    let mut game_state: &mut GameState;

    // -------- INITIALIZE GAME STATE IF NEEDED ----------
    unsafe {
        match &mut GAME_STATE_HOLDER {
            None => {
                spritesheet::Sprite::init_all_sprites();
                let mut new_game_state = GameState::new();
                for _ in 0..20 {
                    new_game_state.rng.next_for_worldgen();
                }
                
                new_game_state.regenerate_map();
                GAME_STATE_HOLDER = Some(new_game_state);
            }
            Some(_) => {}
        }
        match &mut GAME_STATE_HOLDER {
            Some(game_state_holder) => {
                game_state = game_state_holder;
            }
            None => unreachable!(),
        }
    }

    // ----------- UPDATE TIMER AND PLAY BGM -----------
    game_state.song_timer += 1;
    play_bgm(game_state.song_timer, &SONGS[game_state.song_idx]);


    // let mut player_idx: u8 = 0b0;
    let player_idx: u8 = 0b0;

    // UPDATE WHICH PLAYER WE'RE PLAYING IN NETPLAY
    // unsafe {
    //     // If netplay is active
    //     if *NETPLAY & 0b100 != 0 {
    //         player_idx = *NETPLAY & 0b011;
    //     // Render the game from player_idx's perspective
    //     } else {
    //     }
    // }

    // SET CAMERA POSITION
    match &mut game_state.players[player_idx as usize] {
        OptionallyEnabledPlayer::Disabled => {}
        OptionallyEnabledPlayer::Enabled(player) => {
            game_state.camera.current_viewing_x_target = num::clamp(
                player.character.x_pos - center_x,
                X_LEFT_BOUND as f32,
                X_RIGHT_BOUND as f32,
            );
            game_state.camera.current_viewing_y_target = num::clamp(
                player.character.y_pos - center_y,
                Y_LOWER_BOUND as f32,
                Y_UPPER_BOUND as f32,
            );
        }
    }

    game_state.camera.slew();

    // ------------- POLL INPUT ---------------

    // let [btns_pressed_this_frame, gamepads] = get_inputs_this_frame();

    // CHECK IF WE NEED TO FREEZE CHARACTERS / GAMEPLAY ON SCREEN
    let mut showing_modal = false;
    match &game_state.game_mode {
        GameMode::NormalPlay(play_mode) => {
            

            match play_mode {
                NormalPlayModes::MainGameplay => {
                    // handle player inputs here
                    game_state.countdown_paused = false;
                }
                NormalPlayModes::HoverModal(_) => {
                    showing_modal = true;
                    game_state.countdown_paused = true;
                }
            }
        },
        _ => {}
    }
    // ON TITLE SCREEN, MOVE PLAYER 1 BASED ON TIME
    
    // CHECK IF CHARACTERS / CATS ARE COLLIDING
    if !showing_modal {
        check_entity_collisions(&mut game_state);
    }
    
    // PREPARE TO RENDER THE MAP & ENTITIES
    // unsafe {
    //     *PALETTE = spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx];
    // }
    set_palette(&Pallette{
        main_kitty: spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx][3],
        pigs_lizards: spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx][2],
        foreground: spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx][1],
        background: spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx][0],
    });
    // unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }

    // MOVE AND RENDER THE PLAYERS 
    {
        let optional_players: &mut [OptionallyEnabledPlayer; 4] = &mut game_state.players;

        for (i, optional_player) in &mut optional_players.iter_mut().enumerate() {


            let mut input = match false { // showing_modal {
                false => gamepads[i],
                true => 0,
            };
            if i == 0 {
                match game_state.game_mode {
                    GameMode::StartScreen => {
                        let mut move_n = (((game_state.song_timer / 10) * 31) % 29) as u8;
                        move_n &= !(BUTTON_LEFT | BUTTON_RIGHT);
                        input = move_n;
                        match move_n {
                            0..=2 => {
                                input |= BUTTON_LEFT;
                            },
                            3..=6=> {
                                input |= BUTTON_RIGHT;
                            }
                            _ => {}
                        }
                    },
                    _ => {},
                }
            }
            

            update_pos(
                &game_state.map,
                MovingEntity::OptionalPlayer(optional_player),
                input,
                game_state.godmode,
                &mut game_state.clouds,
            );
            
        

            drawcharacter(
                &game_state.camera,
                MovingEntity::OptionalPlayer(optional_player),
                blit_sub
            );
        }
    }

   


    // CREATE INPUTS FOR NPCS
    let inputs: &mut [u8; MAX_N_NPCS] = unsafe { &mut NPC_INPUTS };
    let l;
    {
        l = game_state.npcs.len();
    }
    for i in 0..l {
        let rng = &mut game_state.rng;
        let rand_val = (rng.next_for_input() % 255) as u8;
        let current_npc = &mut game_state.npcs[i];
        let mut use_rng_input = false;
        match current_npc.following_i {
            None => {
                use_rng_input = true;
            }
            Some(p_i) => {
                let the_opt_player = &game_state.players[p_i as usize];
                if let OptionallyEnabledPlayer::Enabled(p) = the_opt_player {
                    let p_bound = get_bound_of_character(&p.character);
                    let npc_bound: AbsoluteBoundingBox<i32, u32> =
                        get_bound_of_character(&current_npc);
                    let needs_teleport;
                    {
                        // teleportAyh-shon if needed
                        const TELEPORT_AXIS_MIN_DIST: u32 = 160;
                        if p_bound.x.abs_diff(npc_bound.x) > TELEPORT_AXIS_MIN_DIST
                            || p_bound.y.abs_diff(npc_bound.y) > TELEPORT_AXIS_MIN_DIST
                        {
                            needs_teleport = true
                        } else {
                            needs_teleport = false
                        }
                    }

                    if needs_teleport {
                        current_npc.x_pos = p_bound.x as f32;
                        current_npc.y_pos = p_bound.y as f32;
                        current_npc.x_vel = 0.0;
                        current_npc.y_vel = 0.0;
                    } else {
                        if rng.next_for_input() % 10 > 1 {
                            inputs[i] = 0;

                            // if current_npc.x_pos + (npc_bound.width as f32) < p.x_pos {
                            // else if current_npc.x_pos > p.x_pos + p_bound.width as f32 {

                            // make NPCs tryhard when they're not in the same Y to get to exact x position to help with climbing
                            let mut tryhard_get_to_0: bool = true;
                            let ch = &p.character;
                            // fall by doing nothing
                            if current_npc.y_pos + (npc_bound.height as f32) < ch.y_pos {
                            } else if current_npc.y_pos > ch.y_pos + p_bound.height as f32 {
                                inputs[i] |= BUTTON_1;
                            } else {
                                tryhard_get_to_0 = false;
                            }

                            if tryhard_get_to_0 {
                                if current_npc.x_pos < ch.x_pos {
                                    inputs[i] |= BUTTON_RIGHT;
                                } else if current_npc.x_pos > ch.x_pos {
                                    inputs[i] |= BUTTON_LEFT;
                                }
                            } else {
                                if current_npc.x_pos + (npc_bound.width as f32) < ch.x_pos {
                                    inputs[i] |= BUTTON_RIGHT;
                                } else if current_npc.x_pos > ch.x_pos + p_bound.width as f32
                                {
                                    inputs[i] |= BUTTON_LEFT;
                                }
                            }
                        } else {
                            use_rng_input = true;
                        }
                    }
                } else {
                    use_rng_input = false;
                }
            }
        }

        if use_rng_input {
            if rand_val < 20 {
                inputs[i] = 0x10;
            } else if rand_val < 40 {
                inputs[i] = 0x20;
            } else if rand_val < 42 {
                inputs[i] = BUTTON_1;
            } else {
                inputs[i] = 0x0;
            }
        }
        


    }

    // MOVE NPCS
    for (i, npc) in game_state.npcs.iter_mut().enumerate() {
        update_pos(
            &game_state.map,
            MovingEntity::NPC(npc),
            inputs[i],
            game_state.godmode,
            &mut game_state.clouds,
        );
    }

    // DRAW NPCS
    for npc in game_state.npcs.iter_mut() {
        drawcharacter(
            &game_state.camera,
            MovingEntity::NPC(npc),
            blit_sub
        );
    }

 
    // ------ RENDER THE MAP -----------
    drawmap(&game_state, blit_sub, sw, sh);

    // UPDATE CLOUDS
    Cloud::update_clouds(&mut game_state.clouds);

    // DRAW CLOUDS
    for cloud in game_state.clouds.iter() {
        let cam: &Camera = &game_state.camera;
        let cloud_sprite: &spritesheet::Sprite = spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Cloud);
        blit_sub(
            Spritesheet::Main,
            (cloud.x - cam.current_viewing_x_offset) as i32,
            (cloud.y - cam.current_viewing_y_offset) as i32,
            cloud_sprite.frames[0].width as u32,
            cloud_sprite.frames[0].height as u32,
            cloud_sprite.frames[0].start_x as u32,
            cloud_sprite.frames[0].start_y as u32,
            BlitSubFlags { 
                flip_x: cloud.vx > 0.,
                flip_y: cloud.vy < 0.
            }
        );
    }

    // just draw a spriteframe at a location. Put a colored layer behind it, like layertext() does.
    fn draw_spriteframe (spriteframe: &spritesheet::SpriteFrame, x: i32, y: i32, blit_sub: &mut BlitSubFunc) {
        let cf = spriteframe;
        // for (xx, yy, colors) in [(x, y, 0x1111), (x+1, y+1, 0x1111), (x, y, spritesheet::KITTY_SPRITESHEET_DRAW_COLORS)] {

        for (xx, yy, _colors) in [(x, y, spritesheet::KITTY_SPRITESHEET_DRAW_COLORS)] {
            // unsafe {*DRAW_COLORS = colors}
            blit_sub(
                Spritesheet::Main,
                xx,
                yy,
                cf.width as u32,
                cf.height as u32,
                cf.start_x as u32,
                cf.start_y as u32,
                BlitSubFlags { flip_x: false, flip_y: false }
            );
        }
        
    }

    // Depending on what gamemode we're in, we do different update steps.
    match &mut game_state.game_mode {
        GameMode::NormalPlay(play_mode) => {
            // Draw blur sections for the status bars on the bottom and top of the screen.
            draw_modal_bg(
                &AbsoluteBoundingBox {
                    x: -1.0,
                    y: 0.0,
                    width: sw as f32 + 2.0,
                    height: 10.0,
                },
                0,
                &DrawColor::Background,
                line, 
                rect
            );

            draw_modal_bg(
                &AbsoluteBoundingBox {
                    x: -1.0,
                    y: sh as f32 - 10.,
                    width: sw as f32 + 2.0,
                    height: 10.0,
                },
                0,
                &DrawColor::Background,
                line,
                rect,
            );



            // COUNT THE NUMBER OF NPCS THAT ARE FOLLOWING PLAYERS
            let current_found_npcs: u32 = game_state.npcs
                .iter()
                .fold(0, |acc, e| acc + match e.following_i {None => 0, Some(_) => 1});

            // COMPUTE SCORE, LEVEL, # KITTIES (used later either in modal or normal screen)
            let world_level_text = &format!["W{}-L{}", ((game_state.difficulty_level - 1) / LEVELS_PER_MOOD as u32) + 1, ((game_state.difficulty_level - 1) % LEVELS_PER_MOOD as u32) + 1];
            let score_text = match game_state.settings.run_type {
                game::game_state::RunType::Random => {
                    format!["Sc: {}p", game_state.score]
                },
                game::game_state::RunType::Speedrun(n) => {
                    format!["Sd.{}: {}s", n, game_state.speedrun_timer_msec/ 60]
                },
            };
            let found_kitties_text = &format![
                "{:<5} {:<3}", &format!["{:.2}/{:.2}", current_found_npcs, game_state.total_npcs_to_find],
                game_state.countdown_timer_msec as u32 / 60
            ];

            // UPDATE & DRAW POPUPS
            {
                let popup_texts_rb: &mut PopTextRingbuffer = &mut game_state.popup_text_ringbuffer;


                popup_texts_rb.update_popup_positions();
                
                

                let camera = game_state.camera;
                for popup in popup_texts_rb.texts.iter() {
                    match popup {
                        Some(p) => {
                            const T_BEFORE_BLINK: u32 = 60;
                            if p.duration_timer < T_BEFORE_BLINK || p.duration_timer % 6 < 3 {
                                let (dx, dy) = ((p.x_pos - camera.current_viewing_x_offset) as i32, (p.y_pos - camera.current_viewing_y_offset) as i32);
                                layertext(&p.text, dx, dy, text_str);
                                match p.icon {
                                    PopupIcon::None => {},
                                    PopupIcon::Clock => {
                                        draw_spriteframe( &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock).frames[0], dx, dy-1, blit_sub)
                                    }
                                    PopupIcon::CatHead => {
                                        draw_spriteframe( &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::CatHead).frames[0], dx+1, dy+1, blit_sub)
                                    },
                                    PopupIcon::DownArrow => {
                                        // text_bytes(&[b'\x87'], dx+40, dy);
                                        text_str("down", dx+32, dy, &DrawColor::MainKitty)
                                    }
                                }
                            }
                        },
                        None => {

                        }
                    }
                }
            }

            // USE ABILITY CARDS
            if !showing_modal {
                for (p_i, pr) in game_state.players.iter_mut().enumerate() {
                    match pr {
                        OptionallyEnabledPlayer::Enabled(p) => {
                            if !showing_modal && btns_pressed_this_frame[p_i] & BUTTON_2 != 0 {
                                let res = p.card_stack.try_use_cards();
                                let added_t;
                                let popup_t: Option<String>;
                                let popup_icon: PopupIcon;
                                match res {
                                    game::ability_cards::AbilityCardUsageResult::NothingHappened => {
                                        added_t = 0;
                                        popup_t = None;
                                        popup_icon = PopupIcon::None;
                                    },
                                    game::ability_cards::AbilityCardUsageResult::GainedTime(t) => {
                                        added_t = t;
                                        popup_t = Some(format![" +{}", t]);
                                        popup_icon = PopupIcon::Clock;
                                    },
                                    game::ability_cards::AbilityCardUsageResult::EnabledFlyAndTime(t) => {
                                        if p.character.can_fly {
                                            added_t = t;
                                            popup_t = Some(format![" +{}", t]);
                                            popup_icon = PopupIcon::Clock;
                                        } else {
                                            p.character.can_fly = true;
                                            added_t = t - 10;
                                            popup_t = Some("fly!".to_string());
                                            popup_icon = PopupIcon::None;
                                        }
                                        
                                    },
                                    game::ability_cards::AbilityCardUsageResult::EnabledWarpAndTime(t) => {
                                        if p.character.warp_ability == WarpAbility::CannotWarp {
                                            p.character.warp_ability = WarpAbility::CanWarp(WarpState::Charging(0));
                                            added_t = t - 10;
                                            popup_t = Some("hold   : warp".to_string());
                                            popup_icon = PopupIcon::DownArrow;
                                        } else {
                                            added_t = 10;
                                            popup_t = Some(format![" +{}", t]);
                                            popup_icon = PopupIcon::Clock;
                                        }
                                    }
                                }
                                match popup_t {
                                    Some(pt) => {
                                        // spawn some clouds
                                        for dir in [(1.0, 0.0), (0.5, 0.86), (-0.5, 0.86), (-1.0, 0.0), (-0.5, -0.86), (0.5, -0.86)] {
                                            const CARD_CLOUD_SPEED: f32 = 4.0;

                                            let vx = CARD_CLOUD_SPEED * dir.0;
                                            let vy = CARD_CLOUD_SPEED * dir.1;
                                            Cloud::try_push_cloud(&mut game_state.clouds, p.character.x_pos + 2.0, p.character.y_pos + 3.0, vx, vy);

                                        }
                                        game_state.popup_text_ringbuffer.add_new_popup(p.character.x_pos - 14.0, p.character.y_pos, pt, popup_icon);
                                    }
                                    _ => {}
                                }
                                game_state.countdown_timer_msec += added_t * 60;
                                game_state.countdown_timer_msec = game_state.countdown_timer_msec.min(100 * 60 - 1);
                                game_state.score += added_t * 60;
                            }
                        },
                        OptionallyEnabledPlayer::Disabled => {},
                    }
                }
                
            }

            // MOVE ABILITY CARD POSITIONS
            match &mut game_state.players[player_idx as usize] {
                OptionallyEnabledPlayer::Enabled(p) => {
                    for (i, card) in p.card_stack.cards.iter_mut().enumerate() {
                        match card {
                            Some(c) => {
                                c.target_x = (sw as usize - 80 + 15 * i) as f32;
                                c.target_y = 1.0;
                            },
                            None => {}
                        }
                    }
                    p.card_stack.move_cards();
                },
                OptionallyEnabledPlayer::Disabled => {}
            }

            
            // DRAW ABILITY CARDS
            // unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
            match &game_state.players[player_idx as usize] {
                OptionallyEnabledPlayer::Enabled(p) => {
                    for card in p.card_stack.cards.iter() {
                        match &card {
                            
                            Some(c) => {
                                // trace(&format!["{}", i]);
                                blit_sub(
                                    Spritesheet::Main,
                                    c.x_pos as i32,
                                    c.y_pos as i32,
                                    c.sprite.frames[0].width as u32,
                                    c.sprite.frames[0].height as u32,
                                    c.sprite.frames[0].start_x as u32,
                                    c.sprite.frames[0].start_y as u32,
                                    BlitSubFlags { flip_x: false, flip_y: false },
                                );
                            },
                            None => {},
                        }
                        
                    }
                },
                OptionallyEnabledPlayer::Disabled => {},
            }

            // SHOW MODAL DIALOGS
            if showing_modal {
                match play_mode {
                    NormalPlayModes::MainGameplay => {
                        unreachable!()
                    }
                    NormalPlayModes::HoverModal(m) => {
                        let mut options_ready_to_select: bool = false;
                        
                        let ready_to_show_text;
                        {
                            let actual_position: &mut AbsoluteBoundingBox<f32, f32> = &mut m.actual_position;
                            let target_position: &mut AbsoluteBoundingBox<i32, u32> = &mut m.target_position;

                            const SPEED: f32 = 0.15;
                            const TOL: f32 = 10.0;

                            let real_tpy = target_position.y + (4f32 * num::Float::sin(game_state.song_timer as f32 * 0.05f32)) as i32;

                            actual_position.x += (target_position.x as f32 - actual_position.x) * SPEED;
                            actual_position.y += (real_tpy as f32 - actual_position.y) * SPEED;
                            actual_position.width += (target_position.width as f32 - actual_position.width) * SPEED;
                            actual_position.height += (target_position.height as f32 - actual_position.height) * SPEED;

                            ready_to_show_text = (actual_position.width - target_position.width as f32).abs() < TOL;

                            draw_modal_bg(&actual_position, 1, &DrawColor::Foreground, line, rect);
                        
                        }

                        let mut text_timer = 0;
                        {
                            m.timer += 1;
                        }
                        const INTERACTIVE_DELAY: u32 = 60;
                        if ready_to_show_text && m.timer >= INTERACTIVE_DELAY {
                            {
                                options_ready_to_select = true;
                                text_timer = m.timer;

                            }
                        }
                        
                        let mut modal_text = |st: &str, x, y| {
                            // unsafe {*DRAW_COLORS = 0x0002}
                            text_str(st, m.actual_position.x as i32 + x, m.actual_position.y as i32 + y, &DrawColor::Foreground);
                        };

                        let modal_offs = |x: i32, y: i32| {
                            (m.actual_position.x as i32 + x, m.actual_position.y as i32 + y)
                        };
                        
                        

                        if ready_to_show_text {
                            // let cursor_opt: u8;
                            let mut btn_pressed: bool = false;
                            if options_ready_to_select {
                                // cursor_opt = *option;
                                if btns_pressed_this_frame[0] & (BUTTON_1 | BUTTON_2) != 0 {
                                    btn_pressed = true
                                }
                            }
                            match m.menu_type {
                                MenuTypes::WonLevel => {
                                    const BLINK_START: u32 = 50;
                                    const BLINK_TITLE_PERIOD: u32 = 17;
                                    if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD) % 2 == 0 {
                                        // modal_text("Found!!", 12, 15);
                                        modal_text(world_level_text, 16, 12);
                                        modal_text("Clear!", 16, 22);


                                    }

                                    match btn_pressed {
                                        true => {
                                            game_state.difficulty_level += 1;
                                            // game_state.game_mode =
                                            //     GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                            game_state.game_mode =
                                                GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                                                    AbsoluteBoundingBox {
                                                        x: center_x as i32 - 35,
                                                        y: center_y as i32 - 40,
                                                        width: 70,
                                                        height: 50,
                                                    },
                                                    MenuTypes::StartLevel,
                                                )));
                                            game_state.regenerate_map();
                                        }
                                        _ => {}
                                    }
                                },
                                MenuTypes::StartLevel => {
                                    modal_text(world_level_text, 16, 12);
                                    modal_text("Start!", 16, 22);
                                    modal_text(&format!["+{}", game_state.countdown_and_score_bonus / 60], 33, 35);
                                    let (xx, yy) = modal_offs(25, 34);
                                    draw_spriteframe( &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock).frames[0], xx, yy, blit_sub);

                                    let mut start_normal_play = false;
                                    if text_timer > 100 {
                                        start_normal_play = true;
                                    }


                                    match btn_pressed {
                                        true => {
                                            start_normal_play = true; 
                                        }
                                        _ => {}
                                    }
                                    if start_normal_play {
                                        game_state.game_mode =
                                            GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                    }
                                },
                                MenuTypes::Done => {
                                    const BLINK_START: u32 = 50;
                                    const BLINK_TITLE_PERIOD: u32 = 17;
                                    if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD) % 2 == 0 {
                                        modal_text("Time's Up!", 20, 14);
                                    }

                                    modal_text(&format!["End: {}", world_level_text], 8, 30);
                                    modal_text(&score_text, 8, 40);
                                    

                                    match btn_pressed {
                                        true => {
                                            game_state.difficulty_level = START_DIFFICULTY_LEVEL;
                                            game_state.game_mode = GameMode::StartScreen;
                                        }
                                        _ => {}
                                    }
                                },
                                MenuTypes::WonGame => {
                                    const BLINK_START: u32 = 50;
                                    const BLINK_TITLE_PERIOD: u32 = 17;
                                    if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD) % 2 == 0 {
                                        modal_text("YOU WON!!!", 20, 14);
                                    }

                                    modal_text(&format!["End: {}", world_level_text], 8, 30);
                                    modal_text(&score_text, 8, 40);
                                    match game_state.settings.run_type {
                                        RunType::Random => {
                                            modal_text(&format!["Time: {}s", game_state.speedrun_timer_msec / 60], 8, 50);
                                        },
                                        _ => {}
                                    }

                                    match btn_pressed {
                                        true => {
                                            game_state.difficulty_level = START_DIFFICULTY_LEVEL;
                                            game_state.game_mode = GameMode::StartScreen;
                                        }
                                        _ => {}
                                    }
                                },
                                MenuTypes::StartGameMessage => {
                                    modal_text("-- GOAL --", 30, 10);
                                    modal_text("Find all the", 20, 25);
                                    modal_text("kitties in time!", 10, 40);
                                    modal_text("-- CONTROLS --", 14, 100);



                                    modal_text("     to move,", 24, 114); 
                                    modal_text(" =jump,  =card", 16, 126);
                                    let (xx, yy) = modal_offs(0, 0);
 
                                    if game_state.song_timer % 30 >= 15 {
                                        // unsafe {*DRAW_COLORS = 0x0004}
                                        // text_bytes(&[b'\x84'], xx+32, yy+114);
                                        // text_bytes(&[ b'\x85'], xx+48, yy+114);
                                        // text_bytes(&[b'\x80'], xx+15, yy+126);
                                        // text_bytes(&[ b'\x81'], xx+79, yy+126);
                                        // text_str("<", xx+32, yy+114, &DrawColor::MainKitty);
                                        // text_str(">", xx+48, yy+114, &DrawColor::MainKitty);
                                        // text_str("x", xx+15, yy+126, &DrawColor::MainKitty);
                                        // text_str("z", xx+79, yy+126, &DrawColor::MainKitty);
                                    }
                                    
                                    draw_spriteframe( &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::CatHead).frames[0], xx+20, yy+62, blit_sub);

                                    modal_text(" = # kittes", 28, 62);
                                    draw_spriteframe( &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock).frames[0], xx+20, yy+78, blit_sub);

                                    modal_text(" = time left", 28, 78);
                                    match btn_pressed {
                                        true => {
                                            game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                        }
                                        _ => {}
                                    }
                                }
                            }     
                        }
                    }
                }
            } else {

                // HELP TEXT AT START OF GAME
                if game_state.difficulty_level == 1 && game_state.countdown_timer_msec == COUNTDOWN_TIMER_START - 1 && game_state.tutorial_text_counter == 0 {
                    game_state.tutorial_text_counter += 1;
                    game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                        AbsoluteBoundingBox {
                            x: center_x as i32 - 70,
                            y: center_y as i32 - 70,
                            width: 140,
                            height: 140,
                        },
                        MenuTypes::StartGameMessage
                    )));
                }
                

                // ------- LEVEL WIN CONDITION -----------
                if game_state.total_npcs_to_find == current_found_npcs {
                    if game_state.difficulty_level == FINAL_LEVEL {
                        game_state.game_mode =
                        GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                            AbsoluteBoundingBox {
                                x: 25,
                                y: 35,
                                width: 110,
                                height: 65,
                            },
                            MenuTypes::WonGame
                        )));
                        game_state.song_idx = 0;
                    } else {
                        game_state.game_mode =
                        GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                            AbsoluteBoundingBox {
                                x: center_x as i32 - 40,
                                y: center_y as i32 - 40,
                                width: 80,
                                height: 40,
                            },
                            MenuTypes::WonLevel
                        )));
                        game_state.song_idx = 0;
                    }

                    game_state.song_timer = 0;
                }

                // PROGRESS TIME, CHECK FOR GAME END
                if !game_state.countdown_paused {
                    game_state.speedrun_timer_msec += 1;
                    game_state.countdown_timer_msec -= 1;
            
                    // ---- LOSE CONDITION ----
                    if game_state.countdown_timer_msec <= 0 {
            
                        game_state.song_idx = 0;
            
                        game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                            AbsoluteBoundingBox {
                                x: 15,
                                y: 50,
                                width: 130,
                                height: 60,
                            },
                            MenuTypes::Done
                        )));
                    }
                }



                // DRAW SCORE, LEVEL, # KITTIES during normal play
                layertext(world_level_text, 0, sh as i32 + BOTTOM_UI_TEXT_Y_OFFSET, text_str);
                layertext(&score_text, 60, sh as i32 + BOTTOM_UI_TEXT_Y_OFFSET, text_str);
                layertext(found_kitties_text, 9, TOP_UI_TEXT_Y, text_str);

                draw_spriteframe( &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock).frames[0], 48, TOP_UI_TEXT_Y - 1, blit_sub);
                draw_spriteframe( &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::CatHead).frames[0], 1, TOP_UI_TEXT_Y + 1, blit_sub);

                
            }
        }
        GameMode::StartScreen => {
            
            
            // SETUP TITLE MUSIC AND COLORS
            game_state.song_idx = 1;
            // unsafe { *DRAW_COLORS = 0x0002 }

            // SHOW TITLE-SCREEN SUBTEXT
            if game_state.song_timer >= TIMER_INTERACTIVE_START {
                draw_modal_bg(
                    &AbsoluteBoundingBox {
                        x: 15.0,
                        y: 105.0,
                        width: 130.0,
                        height: 40.0,
                    },
                    0,
                    &DrawColor::Background,
                    line,
                    rect,
                );
                // unsafe{*DRAW_COLORS = 0x0002};
                if game_state.song_timer % 30 >= 15 {
                    text_str("Any key: play", center_x as i32 - 50, 110, &DrawColor::MainKitty);
                }
                
                text_str("by CanyonTurtle", center_x as i32 - 55, 125, &DrawColor::MainKitty);
                text_str(" & BurntSugar  ", center_x as i32 - 50, 135, &DrawColor::MainKitty);
                text_str(&format!["ver. {}.{}.{}", MAJOR_VERSION, MINOR_VERSION, INCR_VERSION], 40, 150, &DrawColor::MainKitty);
                if btns_pressed_this_frame[0] != 0 {
                    // game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                    game_state.game_mode = GameMode::SelectScreen(SelectSetup{current_selection: SelectMenuFocuses::RunType});
                    // game_state.regenerate_map();
                }
            }
            

            render_title(&game_state, sw as i32 / 2 - 76, TITLE_Y, blit_sub);
            game_state.rng.next_for_input();
            
            // trace("updated positions");
            // unsafe { *DRAW_COLORS = 0x1112 }
            
        },
        GameMode::SelectScreen(select_setup) => {

            const BOX_WIDTH: i32 = 120;
            let box_margin = (sw as i32 - BOX_WIDTH) / 2;
            const BOX_HEIGHT: i32 = 60;

            const RUN_TYPE_Y: i32 = 66;
            // const DIFFICULTY_Y: i32 = 33;
            // const CHARACTER_Y: i32 = 46;
            const START_Y: i32 = 130;
            let start_x: i32 = center_x as i32 - 25;

            // const START_WIDTH: i32 = 60;
            // const START_HEIGHT: i32 = 19;

            const SETTING_GROUP_INLAY_DIST: i32 = 5;

            // let mut selected_box_dims = (0, 0, 0, 0);

            fn draw_selected_box(dims: (i32, i32, i32, i32), style: u8, color: &DrawColor, line: &mut LineFunc, rect: &mut RectFunc) {
                draw_modal_bg(&AbsoluteBoundingBox{x: dims.0 as f32, y: dims.1 as f32, width: dims.2 as f32, height: dims.3 as f32}, style, color, line, rect);
            }

            // draw background for menus
            draw_modal_bg(&AbsoluteBoundingBox{x: 0f32, y: 0f32, width: 159f32, height: 159f32}, 0, &DrawColor::Foreground, line, rect);
            draw_selected_box((box_margin, RUN_TYPE_Y, BOX_WIDTH, BOX_HEIGHT), 0, &DrawColor::Foreground, line, rect);

            // draw options that get overdrawn later if they're not selected
            // layertext("Run Type", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST);
            // layertext("Difficulty", box_margin + SETTING_GROUP_INLAY_DIST, DIFFICULTY_Y + SETTING_GROUP_INLAY_DIST);
            // layertext("Character", box_margin + SETTING_GROUP_INLAY_DIST, CHARACTER_Y + SETTING_GROUP_INLAY_DIST);

            // if btns_pressed_this_frame[0] & BUTTON_DOWN != 0 {
            //     select_setup.current_selection = match select_setup.current_selection {
            //         SelectMenuFocuses::RunType => SelectMenuFocuses::StartGameBtn,
            //         // SelectMenuFocuses::Difficulty => SelectMenuFocuses::CharacterSelect,
            //         // SelectMenuFocuses::CharacterSelect => SelectMenuFocuses::StartGameBtn,
            //         SelectMenuFocuses::StartGameBtn => SelectMenuFocuses::StartGameBtn,
            //     }   
            // }

            // if btns_pressed_this_frame[0] & BUTTON_UP != 0 {
            //     select_setup.current_selection = match select_setup.current_selection {
            //         SelectMenuFocuses::RunType => SelectMenuFocuses::RunType,
            //         // SelectMenuFocuses::Difficulty => SelectMenuFocuses::RunType,
            //         // SelectMenuFocuses::CharacterSelect => SelectMenuFocuses::Difficulty,
            //         SelectMenuFocuses::StartGameBtn => SelectMenuFocuses::RunType,
            //     }   
            // }

            match select_setup.current_selection {
                SelectMenuFocuses::RunType => {
                    if btns_pressed_this_frame[0] & (BUTTON_RIGHT | BUTTON_LEFT) != 0 {
                        game_state.settings.run_type = match game_state.settings.run_type {
                            game::game_state::RunType::Random => game::game_state::RunType::Speedrun(0),
                            game::game_state::RunType::Speedrun(_) => game::game_state::RunType::Random,
                        }   
                    }
                    // draw box around run type
                    draw_selected_box((box_margin, RUN_TYPE_Y, BOX_WIDTH, BOX_HEIGHT), 1, &DrawColor::MainKitty, line, rect);
                    // layertext("Run Type", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST);

                    if game_state.song_timer % 30 >= 15 {
                        // unsafe {*DRAW_COLORS = 0x0004}
                        // text_bytes(&[b'\x85'], 132, 72);
                        // text_bytes(&[b'\x80'], 45, 136);
                        text_str(">", center_x as i32 + 52, 72, &DrawColor::MainKitty);
                        text_str("x", start_x - 10, 136, &DrawColor::MainKitty);
                    }

                    if btns_pressed_this_frame[0] & (BUTTON_2) != 0 {
                        if let game::game_state::RunType::Speedrun(n) = game_state.settings.run_type {
                            game_state.settings.run_type = game::game_state::RunType::Speedrun(n + 1);
                        }
                    }

                    layertext("Start!", start_x + SETTING_GROUP_INLAY_DIST + 3, START_Y + SETTING_GROUP_INLAY_DIST + 1, text_str);

                    if btns_pressed_this_frame[0] & BUTTON_1 != 0 {
                        game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                        if let game::game_state::RunType::Speedrun(n) = game_state.settings.run_type {
                            game_state.rng = GameRng::FixedSeed(Rng::new_from_seed(n), Rng::new_from_seed(n));
                        }
                        game_state.regenerate_map();
                    }
                },
                // SelectMenuFocuses::Difficulty => {
                //     // draw box around difficulty
                //     draw_selected_box((box_margin, DIFFICULTY_Y, BOX_WIDTH, BOX_HEIGHT));
                //     layertext("Difficulty", box_margin + SETTING_GROUP_INLAY_DIST, DIFFICULTY_Y + SETTING_GROUP_INLAY_DIST);
                // },
                // SelectMenuFocuses::CharacterSelect => {
                //     draw_selected_box((box_margin, CHARACTER_Y, BOX_WIDTH, BOX_HEIGHT));
                //     layertext("Character", box_margin + SETTING_GROUP_INLAY_DIST, CHARACTER_Y + SETTING_GROUP_INLAY_DIST);

                // },
                // SelectMenuFocuses::StartGameBtn => {
                //     draw_selected_box((START_X, START_Y, START_WIDTH, START_HEIGHT), 1, 0x0004);


                //     if game_state.song_timer % 30 >= 15 {
                //         layertext("Start!", START_X + SETTING_GROUP_INLAY_DIST + 3, START_Y + SETTING_GROUP_INLAY_DIST + 1);
                //     }
                // }
            }

            match game_state.settings.run_type {
                game::game_state::RunType::Random => {
                    layertext("Normal Mode", box_margin + SETTING_GROUP_INLAY_DIST + 20, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST, text_str);
                    layertext("Random levels.", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 15, text_str);
                    layertext("Find kitties", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 25, text_str);
                    layertext("in time!", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 35, text_str);

                },
                game::game_state::RunType::Speedrun(n) => {
                    layertext("Seed Mode", box_margin + SETTING_GROUP_INLAY_DIST + 25, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST, text_str);
                    layertext("Fixed maps", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 15, text_str);
                    layertext("For speedruns!", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 25, text_str);
                    layertext(&format![" for seed: {}", n],box_margin + SETTING_GROUP_INLAY_DIST + 1, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 35, text_str);
                    if game_state.song_timer % 30 >= 15 {
                        // unsafe {*DRAW_COLORS = 0x0004}

                        // text_bytes(&[b'\x81'], box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 35);
                        text_str("z", box_margin + SETTING_GROUP_INLAY_DIST, RUN_TYPE_Y + SETTING_GROUP_INLAY_DIST + 35, &DrawColor::MainKitty);

                    }
                },
            }
            

            // Draw box around selection
            // draw_modal_bg(&AbsoluteBoundingBox{x: selected_box_dims.0 as f32, y: selected_box_dims.1 as f32, width: selected_box_dims.2 as f32, height: selected_box_dims.3 as f32}, 1);

            render_title(&game_state, sw as i32 / 2 - 76, TITLE_Y - 8, blit_sub);
            
        }
    }
}
