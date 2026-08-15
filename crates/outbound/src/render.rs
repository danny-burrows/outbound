use crate::item::{Good, Resource};
use crate::rng::Rng;
use crate::sim::Simulation;
use crate::village::PlanState;
use crate::world::World;
use raylib::consts::KeyboardKey::{KEY_A, KEY_LEFT, KEY_R, KEY_RIGHT, KEY_S};
use raylib::prelude::*;

const MAX_BUILDINGS: usize = 100;

const SCREEN_WIDTH: i32 = 860;
const SCREEN_HEIGHT: i32 = 640;
const STEPS_PER_FRAME_DIVISOR: u32 = 10;

/// Keeps the skyline's randomness from shifting the world a seed builds, so the two stay
/// independently reproducible.
const SKYLINE_SALT: u64 = 0x5B1E;

struct Building {
    rect: Rectangle,
    color: Color,
}

#[expect(
    clippy::cast_precision_loss,
    reason = "skyline dimensions are bounded by the ranges below, far inside f32's exact integer range"
)]
fn spawn_skyline(rng: &mut Rng) -> Vec<Building> {
    let mut buildings = Vec::with_capacity(MAX_BUILDINGS);
    let mut spacing = 0.0;
    for _ in 0..MAX_BUILDINGS {
        let height = rng.between(100, 800) as f32;
        let width = rng.between(50, 200) as f32;
        let channel = |rng: &mut Rng| u8::try_from(rng.between(200, 240)).unwrap_or(u8::MAX);
        buildings.push(Building {
            rect: Rectangle::new(
                -6000.0 + spacing,
                (SCREEN_HEIGHT - 130) as f32 - height,
                width,
                height,
            ),
            color: Color::new(channel(rng), channel(rng), channel(rng), 255),
        });
        spacing += width;
    }
    buildings
}

fn handle_input(rl: &RaylibHandle, player: &mut Rectangle, camera: &mut Camera2D) {
    if rl.is_key_down(KEY_RIGHT) {
        player.x += 2.0;
    } else if rl.is_key_down(KEY_LEFT) {
        player.x -= 2.0;
    }

    camera.target = Vector2::new(0.0, 0.0);

    if rl.is_key_down(KEY_A) {
        camera.rotation -= 1.0;
    } else if rl.is_key_down(KEY_S) {
        camera.rotation += 1.0;
    }
    camera.rotation = camera.rotation.clamp(-40.0, 40.0);

    camera.zoom = rl.get_mouse_wheel_move().mul_add(0.05, camera.zoom);
    camera.zoom = camera.zoom.clamp(0.1, 3.0);

    if rl.is_key_pressed(KEY_R) {
        camera.zoom = 1.0;
        camera.rotation = 0.0;
    }
}

fn to_screen((x, y): (i64, i64)) -> (i32, i32) {
    (
        i32::try_from(x).unwrap_or(i32::MAX),
        i32::try_from(y).unwrap_or(i32::MAX),
    )
}

const fn resource_color(resource: Resource) -> Color {
    match resource {
        Resource::Tree => Color::LAWNGREEN,
        Resource::Berry => Color::PURPLE,
        Resource::Stone => Color::GRAY,
    }
}

fn draw_world<D: RaylibDraw>(
    d: &mut D,
    world: &World,
    buildings: &[Building],
    building_site: Rectangle,
    player: Rectangle,
    villager_rect: Rectangle,
) {
    d.draw_rectangle(-6000, 320, 13000, 8000, Color::DARKGRAY);
    d.draw_rectangle_lines_ex(building_site, 1.0, Color::LIMEGREEN);

    for building in buildings {
        d.draw_rectangle_rec(building.rect, building.color);
    }

    let (villager_x, villager_y) = to_screen(world.render_position);
    d.draw_circle(villager_x, villager_y, 3.0, Color::BLUE);

    for item in world.items() {
        let (x, y) = to_screen(item.position);
        d.draw_circle(x, y, 2.0, resource_color(item.resource));
    }

    for &position in &world.logs {
        let (x, y) = to_screen(position);
        d.draw_circle(x, y, 2.0, Color::BROWN);
    }

    d.draw_rectangle_rec(villager_rect, Color::BLUE);
    d.draw_rectangle_rec(player, Color::RED);
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "float to int casts saturate and the axis markers only need pixel precision"
)]
fn draw_camera_axes<D: RaylibDraw>(d: &mut D, camera: &Camera2D) {
    d.draw_line(
        camera.target.x as i32,
        -SCREEN_HEIGHT * 10,
        camera.target.x as i32,
        SCREEN_HEIGHT * 10,
        Color::GREEN,
    );
    d.draw_line(
        -SCREEN_WIDTH * 10,
        camera.target.y as i32,
        SCREEN_WIDTH * 10,
        camera.target.y as i32,
        Color::GREEN,
    );
}

fn draw_hud<D: RaylibDraw>(d: &mut D, state: &PlanState, goal: Option<&str>) {
    d.draw_text("SCREEN AREA", 640, 10, 20, Color::RED);

    d.draw_rectangle(0, 0, SCREEN_WIDTH, 5, Color::RED);
    d.draw_rectangle(0, 5, 5, SCREEN_HEIGHT - 10, Color::RED);
    d.draw_rectangle(SCREEN_WIDTH - 5, 5, 5, SCREEN_HEIGHT - 10, Color::RED);
    d.draw_rectangle(0, SCREEN_HEIGHT - 5, SCREEN_WIDTH, 5, Color::RED);

    d.draw_rectangle(10, 10, 250, 145, Color::SKYBLUE.alpha(0.5));
    d.draw_rectangle_lines(10, 10, 250, 145, Color::BLUE);

    d.draw_text("Free 2d camera controls:", 20, 20, 10, Color::BLACK);
    d.draw_text("- Right/Left to move Offset", 40, 40, 10, Color::DARKGRAY);
    d.draw_text("- Mouse Wheel to Zoom in-out", 40, 60, 10, Color::DARKGRAY);
    d.draw_text("- A / S to Rotate", 40, 80, 10, Color::DARKGRAY);
    d.draw_text(
        "- R to reset Zoom and Rotation",
        40,
        100,
        10,
        Color::DARKGRAY,
    );

    let pursuing = format!("pursuing: {}", goal.unwrap_or("nothing"));
    d.draw_text(&pursuing, 20, 120, 10, Color::BLACK);

    let carrying = format!(
        "carrying: {} wood  {} stone  {} berries",
        state.carrying.count(Good::Wood),
        state.carrying.count(Good::Stone),
        state.carrying.count(Good::Berry),
    );
    d.draw_text(&carrying, 20, 136, 10, Color::BLACK);
}

pub fn run(seed: u64) {
    let (mut rl, thread) = init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Outbound")
        .build();
    rl.set_target_fps(60);

    let mut simulation = Simulation::new(seed);
    let buildings = spawn_skyline(&mut Rng::new(seed ^ SKYLINE_SALT));

    let building_site = Rectangle::new(30.0, 20.0, 25.0, 25.0);
    let villager_rect = Rectangle::new(10.0, 25.0, 10.0, 10.0);
    let mut player = Rectangle::new(400.0, 280.0, 40.0, 40.0);
    let mut frame: u32 = 0;

    let mut camera = Camera2D {
        target: Vector2::new(player.x + 20.0, player.y + 20.0),
        offset: Vector2::new(player.x, player.y),
        rotation: 0.0,
        zoom: 1.0,
    };

    while !rl.window_should_close() {
        // Simulation steps are paced for the eye here; the simulation itself has no
        // opinion about frames.
        if frame.is_multiple_of(STEPS_PER_FRAME_DIVISOR) {
            simulation.advance();
        }
        frame = frame.wrapping_add(1);

        handle_input(&rl, &mut player, &mut camera);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            draw_world(
                &mut d2,
                &simulation.world,
                &buildings,
                building_site,
                player,
                villager_rect,
            );
            draw_camera_axes(&mut d2, &camera);
        }
        draw_hud(&mut d, &simulation.state, simulation.current_goal);
    }
}
