mod actions;
mod goals;
mod goap;
mod item;
mod village;
mod villager;
mod world;

#[cfg(test)]
mod tests;

use crate::actions::VillagerAction;
use crate::goals::Collect;
use crate::goap::{Goal, Plan, PlannerConfig, plan};
use crate::item::{Good, Item, Resource};
use crate::village::{PlanState, Village};
use crate::world::World;
use pathfinding::directed::astar::astar;
use raylib::consts::KeyboardKey::{KEY_A, KEY_LEFT, KEY_R, KEY_RIGHT, KEY_S};
use raylib::prelude::*;
use std::collections::VecDeque;

const MAX_BUILDINGS: usize = 100;
const MAX_TREES: usize = 250;
const MAX_BERRIES: usize = 50;
const MAX_STONE: usize = 25;

const TARGET_WOOD: u8 = 10;
const TARGET_STONE: u8 = 10;
const TARGET_BERRIES: u8 = 10;

const SCREEN_WIDTH: i32 = 860;
const SCREEN_HEIGHT: i32 = 640;
const SPAWN_MAX: i32 = 150;
const TICKS_PER_ACTION: u32 = 10;

struct Building {
    rect: Rectangle,
    color: Color,
}

fn spawn_items(rl: &RaylibHandle) -> Vec<Item> {
    let mut items = Vec::with_capacity(MAX_TREES + MAX_BERRIES + MAX_STONE);
    for (resource, count) in [
        (Resource::Tree, MAX_TREES),
        (Resource::Berry, MAX_BERRIES),
        (Resource::Stone, MAX_STONE),
    ] {
        for _ in 0..count {
            let x: i64 = rl.get_random_value(0..=SPAWN_MAX);
            let y: i64 = rl.get_random_value(0..=SPAWN_MAX);
            items.push(Item::new(resource, (x, y)));
        }
    }
    items
}

fn random_channel(rl: &RaylibHandle) -> u8 {
    u8::try_from(rl.get_random_value::<i32>(200..=240)).unwrap_or(u8::MAX)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "skyline dimensions are bounded by the ranges below, far inside f32's exact integer range"
)]
fn spawn_skyline(rl: &RaylibHandle) -> Vec<Building> {
    let mut buildings = Vec::with_capacity(MAX_BUILDINGS);
    let mut spacing = 0.0;
    for _ in 0..MAX_BUILDINGS {
        let height: i32 = rl.get_random_value(100..=800);
        let width: i32 = rl.get_random_value(50..=200);
        buildings.push(Building {
            rect: Rectangle::new(
                -6000.0 + spacing,
                (SCREEN_HEIGHT - 130 - height) as f32,
                width as f32,
                height as f32,
            ),
            color: Color::new(
                random_channel(rl),
                random_channel(rl),
                random_channel(rl),
                255,
            ),
        });
        spacing += width as f32;
    }
    buildings
}

/// The bottom tier: turn "be over there" into cells to step through.
///
/// Eight-connected at unit cost, which is why [`World::nearest`] measures in Chebyshev
/// distance — that makes the planner's price for a `Travel` the exact length of the walk
/// it defers rather than a guess.
fn walk_to(from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
    let (goal_x, goal_y) = to;
    let Some((mut steps, _)) = astar(
        &from,
        |&(x, y)| {
            (-1..=1)
                .flat_map(move |i| (-1..=1).map(move |j| ((x + i, y + j), 1_u64)))
                .collect::<Vec<_>>()
        },
        |&(x, y)| (x - goal_x).unsigned_abs().max((y - goal_y).unsigned_abs()),
        |&position| position == to,
    ) else {
        return Vec::new();
    };
    steps.reverse();
    steps
}

#[expect(
    clippy::print_stdout,
    reason = "the planner trace is this binary's only diagnostic surface"
)]
fn log_plan(found: &Plan<VillagerAction>) {
    println!(
        "--- plan: {} | cost {} | {} expansions ---",
        found.goal, found.cost, found.expansions
    );
    for action in &found.actions {
        println!("    {action:?}");
    }
}

#[expect(
    clippy::print_stdout,
    reason = "the planner trace is this binary's only diagnostic surface"
)]
fn log_idle(state: &PlanState) {
    println!(
        "--- every goal satisfied, carrying {:?} ---",
        state.carrying
    );
}

/// Runs one plan, refining each abstract action into concrete world changes.
#[derive(Default)]
struct Executor {
    steps: VecDeque<VillagerAction>,
    /// Cells left to step through for the `Travel` in progress.
    walk: Vec<(i64, i64)>,
    /// Latched so the idle notice prints once rather than every tick.
    idle: bool,
}

impl Executor {
    fn advance(
        &mut self,
        world: &mut World,
        state: &mut PlanState,
        goals: &[Box<dyn Goal<Village>>],
        config: &PlannerConfig,
    ) {
        if let Some(position) = self.walk.pop() {
            world.render_position = position;
        } else if let Some(action) = self.steps.pop_front() {
            self.execute(action, world, state);
        } else if let Some(found) = plan(state, world, goals, config) {
            self.idle = false;
            log_plan(&found);
            self.steps = found.actions;
        } else if !self.idle {
            self.idle = true;
            log_idle(state);
        }
    }

    /// Refine one planned action against the real world.
    ///
    /// The plan only asked to be at *a* source; which instance, and the route to it,
    /// is decided here. Splitting it this way is what keeps the search out of the
    /// hundreds of steps a walk actually takes.
    fn execute(&mut self, action: VillagerAction, world: &mut World, state: &mut PlanState) {
        match action {
            VillagerAction::Travel(resource) => {
                if let Some((destination, _)) = world.nearest(world.render_position, resource) {
                    self.walk = walk_to(world.render_position, destination);
                    state.position = destination;
                    state.standing_on = Some(resource);
                    state.logs_underfoot = 0;
                }
            }
            VillagerAction::Harvest(resource) => {
                world.take(resource, state.position);
                state.take(resource);
                state.standing_on = None;
                if let Some(good) = resource.direct_good() {
                    state.carrying.add(good);
                } else {
                    world.drop_log(state.position);
                    state.logs_underfoot = state.logs_underfoot.saturating_add(1);
                }
            }
            VillagerAction::Gather => {
                if world.take_log(state.position) {
                    state.logs_underfoot = state.logs_underfoot.saturating_sub(1);
                    state.carrying.add(Good::Wood);
                }
            }
        }
    }
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

fn draw_hud<D: RaylibDraw>(d: &mut D, state: &PlanState) {
    d.draw_text("SCREEN AREA", 640, 10, 20, Color::RED);

    d.draw_rectangle(0, 0, SCREEN_WIDTH, 5, Color::RED);
    d.draw_rectangle(0, 5, 5, SCREEN_HEIGHT - 10, Color::RED);
    d.draw_rectangle(SCREEN_WIDTH - 5, 5, 5, SCREEN_HEIGHT - 10, Color::RED);
    d.draw_rectangle(0, SCREEN_HEIGHT - 5, SCREEN_WIDTH, 5, Color::RED);

    d.draw_rectangle(10, 10, 250, 133, Color::SKYBLUE.alpha(0.5));
    d.draw_rectangle_lines(10, 10, 250, 133, Color::BLUE);

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

    let carrying = format!(
        "carrying: {} wood  {} stone  {} berries",
        state.carrying.count(Good::Wood),
        state.carrying.count(Good::Stone),
        state.carrying.count(Good::Berry),
    );
    d.draw_text(&carrying, 20, 122, 10, Color::BLACK);
}

pub fn run() {
    let (mut rl, thread) = init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Outbound")
        .build();
    rl.set_target_fps(60);

    let extent = u64::try_from(SPAWN_MAX).unwrap_or(1);
    let mut world = World::new(&spawn_items(&rl), extent);
    let mut state = PlanState::default();
    let buildings = spawn_skyline(&rl);

    let goals: Vec<Box<dyn Goal<Village>>> = vec![
        Box::new(Collect::new(Good::Wood, TARGET_WOOD)),
        Box::new(Collect::new(Good::Stone, TARGET_STONE)),
        Box::new(Collect::new(Good::Berry, TARGET_BERRIES)),
    ];
    let config = PlannerConfig::default();
    let mut executor = Executor::default();
    let mut tick: u32 = 0;

    let building_site = Rectangle::new(30.0, 20.0, 25.0, 25.0);
    let villager_rect = Rectangle::new(10.0, 25.0, 10.0, 10.0);
    let mut player = Rectangle::new(400.0, 280.0, 40.0, 40.0);

    let mut camera = Camera2D {
        target: Vector2::new(player.x + 20.0, player.y + 20.0),
        offset: Vector2::new(player.x, player.y),
        rotation: 0.0,
        zoom: 1.0,
    };

    while !rl.window_should_close() {
        if tick.is_multiple_of(TICKS_PER_ACTION) {
            executor.advance(&mut world, &mut state, &goals, &config);
        }
        tick = tick.wrapping_add(1);

        handle_input(&rl, &mut player, &mut camera);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            draw_world(
                &mut d2,
                &world,
                &buildings,
                building_site,
                player,
                villager_rect,
            );
            draw_camera_axes(&mut d2, &camera);
        }
        draw_hud(&mut d, &state);
    }
}
