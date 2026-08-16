use crate::item::{Good, Resource};
use crate::sim::{Simulation, WORLD_EXTENT};
use crate::villager::Villager;
use crate::world::World;
use raylib::consts::KeyboardKey::{KEY_A, KEY_D, KEY_E, KEY_Q, KEY_R, KEY_S, KEY_W};
use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 860;
const SCREEN_HEIGHT: i32 = 640;
const STEPS_PER_FRAME_DIVISOR: u32 = 3;

/// Empty world units left around the edge when the view is framed.
const VIEW_MARGIN: f32 = 16.0;
/// Screen pixels per second, so panning feels the same however far you are zoomed in.
const PAN_PIXELS: f32 = 320.0;
const ROTATE_DEGREES: f32 = 60.0;
const ZOOM_STEP: f32 = 0.12;
const ZOOM_RANGE: (f32, f32) = (0.5, 40.0);

const BACKGROUND: Color = Color::new(18, 18, 22, 255);
const GROUND: Color = Color::new(32, 33, 38, 255);
const WORLD_EDGE: Color = Color::new(58, 60, 68, 255);

const TREE: Color = Color::new(92, 168, 92, 255);
const STONE: Color = Color::new(150, 155, 165, 255);
const BERRY: Color = Color::new(198, 92, 158, 255);
const LOG: Color = Color::new(140, 96, 56, 255);

const PANEL: Color = Color::new(12, 12, 16, 242);
const PANEL_EDGE: Color = Color::new(48, 50, 58, 255);
const TEXT: Color = Color::new(215, 218, 225, 255);
const TEXT_DIM: Color = Color::new(132, 136, 147, 255);
const TEXT_FAINT: Color = Color::new(94, 98, 108, 255);

const FONT_SIZE: i32 = 16;
const LINE_HEIGHT: i32 = 22;
const PANEL_PAD: i32 = 12;
const PANEL_ORIGIN: i32 = 16;

/// Warm and bright, so villagers read as the moving parts against the greens and greys of
/// what they are after.
const VILLAGER_COLORS: [Color; 6] = [
    Color::new(245, 205, 90, 255),
    Color::new(240, 140, 80, 255),
    Color::new(235, 100, 110, 255),
    Color::new(120, 200, 235, 255),
    Color::new(170, 150, 240, 255),
    Color::new(248, 248, 242, 255),
];

#[expect(
    clippy::cast_precision_loss,
    reason = "screen and world dimensions are small constants, far inside f32's exact integer range"
)]
const fn extent() -> f32 {
    WORLD_EXTENT as f32
}

#[expect(
    clippy::cast_precision_loss,
    reason = "screen and world dimensions are small constants, far inside f32's exact integer range"
)]
fn framed_camera() -> Camera2D {
    let span = extent() + VIEW_MARGIN;
    Camera2D {
        target: Vector2::new(extent() / 2.0, extent() / 2.0),
        offset: Vector2::new(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0),
        rotation: 0.0,
        zoom: SCREEN_WIDTH.min(SCREEN_HEIGHT) as f32 / span,
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "world coordinates are bounded by WORLD_EXTENT, far inside f32's exact integer range"
)]
fn at((x, y): (i64, i64)) -> Vector2 {
    Vector2::new(x as f32, y as f32)
}

fn villager_color(id: usize) -> Color {
    VILLAGER_COLORS
        .get(id % VILLAGER_COLORS.len())
        .copied()
        .unwrap_or(TEXT)
}

fn handle_input(rl: &RaylibHandle, camera: &mut Camera2D) {
    let delta = rl.get_frame_time();
    let pan = PAN_PIXELS * delta / camera.zoom;

    if rl.is_key_down(KEY_W) {
        camera.target.y -= pan;
    }
    if rl.is_key_down(KEY_S) {
        camera.target.y += pan;
    }
    if rl.is_key_down(KEY_A) {
        camera.target.x -= pan;
    }
    if rl.is_key_down(KEY_D) {
        camera.target.x += pan;
    }

    if rl.is_key_down(KEY_Q) {
        camera.rotation = ROTATE_DEGREES.mul_add(-delta, camera.rotation);
    }
    if rl.is_key_down(KEY_E) {
        camera.rotation = ROTATE_DEGREES.mul_add(delta, camera.rotation);
    }

    // Proportional, so one notch covers the same fraction of the view at any zoom.
    let (low, high) = ZOOM_RANGE;
    camera.zoom = rl
        .get_mouse_wheel_move()
        .mul_add(ZOOM_STEP * camera.zoom, camera.zoom)
        .clamp(low, high);

    if rl.is_key_pressed(KEY_R) {
        *camera = framed_camera();
    }
}

fn draw_resource<D: RaylibDraw>(d: &mut D, resource: Resource, position: Vector2) {
    match resource {
        // Shape as well as colour, so the three read apart when zoomed out far enough that
        // everything is a few pixels across.
        Resource::Tree => d.draw_poly(position, 3, 1.8, -90.0, TREE),
        Resource::Stone => d.draw_poly(position, 6, 1.4, 0.0, STONE),
        Resource::Berry => d.draw_circle_v(position, 1.1, BERRY),
    }
}

/// Where a villager is headed, which is the one thing worth seeing that their position alone
/// does not show.
fn draw_intent<D: RaylibDraw>(d: &mut D, villager: &Villager) {
    let standing = at(villager.render_position);
    let heading = at(villager.state.position);
    if heading == standing {
        return;
    }

    let color = villager_color(villager.id);
    d.draw_line_ex(standing, heading, 0.5, color.alpha(0.35));
    d.draw_circle_v(heading, 1.0, color.alpha(0.5));
}

fn draw_villager<D: RaylibDraw>(d: &mut D, villager: &Villager) {
    let standing = at(villager.render_position);
    d.draw_circle_v(standing, 3.1, BACKGROUND);
    d.draw_circle_v(standing, 2.2, villager_color(villager.id));
}

/// A villager's store, which is where every load ends up and so where the shuttling is aimed.
fn draw_home<D: RaylibDraw>(d: &mut D, villager: &Villager) {
    let (x, y) = (at(villager.state.home).x, at(villager.state.home).y);
    let plot = Rectangle::new(x - 2.6, y - 2.6, 5.2, 5.2);
    d.draw_rectangle_lines_ex(plot, 0.7, villager_color(villager.id).alpha(0.55));
}

fn draw_world<D: RaylibDraw>(d: &mut D, world: &World, villagers: &[Villager]) {
    let ground = Rectangle::new(0.0, 0.0, extent(), extent());
    d.draw_rectangle_rec(ground, GROUND);
    d.draw_rectangle_lines_ex(ground, 0.6, WORLD_EDGE);

    for item in world.items() {
        draw_resource(d, item.resource, at(item.position));
    }

    for &position in &world.logs {
        let (x, y) = (at(position).x, at(position).y);
        d.draw_rectangle_rec(Rectangle::new(x - 1.6, y - 0.6, 3.2, 1.2), LOG);
    }

    for villager in villagers {
        draw_home(d, villager);
    }

    // Every intent before any body, so one villager's line is not painted over by another.
    for villager in villagers {
        draw_intent(d, villager);
    }
    for villager in villagers {
        draw_villager(d, villager);
    }
}

fn pursuits(villagers: &[Villager]) -> String {
    Good::ALL
        .into_iter()
        .map(|good| {
            let count = villagers
                .iter()
                .filter(|villager| villager.current_goal == Some(good.label()))
                .count();
            format!("{count} {}", good.label())
        })
        .collect::<Vec<_>>()
        .join("   ")
}

fn draw_hud(d: &mut RaylibDrawHandle<'_>, simulation: &Simulation) {
    let villagers = &simulation.villagers;
    let lines = [
        (
            format!(
                "{} villagers   {} idle",
                villagers.len(),
                simulation.idle_villagers()
            ),
            TEXT,
        ),
        (
            format!(
                "stored   {} wood   {} stone   {} berries   ({} in packs)",
                simulation.stored(Good::Wood),
                simulation.stored(Good::Stone),
                simulation.stored(Good::Berry),
                simulation.in_hand(),
            ),
            TEXT_DIM,
        ),
        (format!("pursuing   {}", pursuits(villagers)), TEXT_DIM),
    ];

    // Sized to its contents, so a wider tally cannot spill past the edge.
    let text_width = lines
        .iter()
        .map(|(line, _)| d.measure_text(line, FONT_SIZE))
        .max()
        .unwrap_or(0);
    let rows = i32::try_from(lines.len()).unwrap_or(0);
    let panel = Rectangle::new(
        f32::from(i16::try_from(PANEL_ORIGIN).unwrap_or(0)),
        f32::from(i16::try_from(PANEL_ORIGIN).unwrap_or(0)),
        f32::from(i16::try_from(text_width + PANEL_PAD * 2).unwrap_or(0)),
        f32::from(i16::try_from(rows * LINE_HEIGHT + PANEL_PAD * 2 - 6).unwrap_or(0)),
    );

    d.draw_rectangle_rec(panel, PANEL);
    d.draw_rectangle_lines_ex(panel, 1.0, PANEL_EDGE);

    for (index, (line, color)) in lines.iter().enumerate() {
        let row = i32::try_from(index).unwrap_or(0) * LINE_HEIGHT;
        d.draw_text(
            line,
            PANEL_ORIGIN + PANEL_PAD,
            PANEL_ORIGIN + PANEL_PAD + row,
            FONT_SIZE,
            *color,
        );
    }

    // Below the framed world, so the least important text covers none of it.
    d.draw_text(
        "WASD pan   Q/E rotate   wheel zoom   R reset",
        PANEL_ORIGIN + PANEL_PAD,
        SCREEN_HEIGHT - 24,
        14,
        TEXT_FAINT,
    );
}

fn draw_frame(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    camera: Camera2D,
    simulation: &Simulation,
) {
    let mut d = rl.begin_drawing(thread);
    d.clear_background(BACKGROUND);
    {
        let mut world_view = d.begin_mode2D(camera);
        draw_world(&mut world_view, &simulation.world, &simulation.villagers);
    }
    draw_hud(&mut d, simulation);
}

pub fn run(seed: u64, population: usize) {
    let (mut rl, thread) = init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Outbound")
        .build();
    rl.set_target_fps(60);

    let mut simulation = Simulation::new(seed, population);
    let mut camera = framed_camera();
    let mut frame: u32 = 0;

    while !rl.window_should_close() {
        // Paced for the eye here; the simulation itself has no opinion about frames.
        if frame.is_multiple_of(STEPS_PER_FRAME_DIVISOR) {
            simulation.advance();
        }
        frame = frame.wrapping_add(1);

        handle_input(&rl, &mut camera);
        draw_frame(&mut rl, &thread, camera, &simulation);
    }
}

/// Advance `warmup` steps, render one frame, and write it to `path`.
///
/// Without this a change to the view can only be checked by a person watching it.
pub fn screenshot(seed: u64, population: usize, warmup: u64, path: &str) {
    let (mut rl, thread) = init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Outbound")
        .build();

    let mut simulation = Simulation::new(seed, population);
    for _ in 0..warmup {
        simulation.advance();
    }

    let camera = framed_camera();
    // Several frames: the earliest land before the window is mapped and come out blank.
    for _ in 0..8 {
        draw_frame(&mut rl, &thread, camera, &simulation);
    }

    // Exporting the framebuffer rather than `take_screenshot`, which resolves its argument
    // against the working directory and so cannot be given an absolute path.
    rl.load_image_from_screen(&thread).export_image(path);
    if std::path::Path::new(path).exists() {
        log::info!("wrote {path}");
    } else {
        log::error!("could not write {path}");
    }
}
