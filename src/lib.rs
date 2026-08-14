mod actions;
mod goals;
mod goap;
mod item;
mod villager;

use crate::actions::{VillageState, VillagerActionEnum};
use crate::goals::{CollectBerries, CollectStone, CollectWood};
use crate::goap::{Action, Goal, plan};
use crate::item::Item;
use crate::villager::Villager;
use pathfinding::directed::astar::astar;
use raylib::consts::KeyboardKey::{KEY_A, KEY_LEFT, KEY_R, KEY_RIGHT, KEY_S};
use raylib::prelude::*;

const MAX_BUILDINGS: usize = 100;
const MAX_TREES: usize = 250;
const MAX_BERRIES: usize = 50;
const MAX_STONE: usize = 25;

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
    for (id, count) in [
        ("tree", MAX_TREES),
        ("berry", MAX_BERRIES),
        ("stone", MAX_STONE),
    ] {
        for _ in 0..count {
            let rx: i64 = rl.get_random_value(0..=SPAWN_MAX);
            let ry: i64 = rl.get_random_value(0..=SPAWN_MAX);
            items.push(Item::new(id.to_string(), (rx, ry)));
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
fn log_plan(steps: &[VillagerActionEnum]) {
    for agent_action in steps {
        println!("---------------------------");
        println!("Action: {agent_action:#?}");
    }
    println!("---------------------------");
}

fn next_plan(
    state: &VillageState,
    goals: &[Box<dyn Goal<VillageState>>],
) -> Vec<VillagerActionEnum> {
    let Some(mut steps) = plan(state.clone(), goals) else {
        return Vec::new();
    };
    log_plan(&steps);
    steps.reverse();
    steps
}

fn advance(
    state: &mut VillageState,
    plan_steps: &mut Vec<VillagerActionEnum>,
    walk: &mut Vec<(i64, i64)>,
    goals: &[Box<dyn Goal<VillageState>>],
) {
    if let Some(position) = walk.pop() {
        state.villager.position = position;
    } else if let Some(agent_action) = plan_steps.pop() {
        if let VillagerActionEnum::MoveToNearestItem(move_action) = &agent_action {
            let destination = move_action.act(state.clone()).villager.position;
            *walk = walk_to(state.villager.position, destination);
        } else {
            *state = agent_action.act(state.clone());
        }
    } else {
        *plan_steps = next_plan(state, goals);
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

fn item_color(id: &str) -> Color {
    match id {
        "tree" => Color::LAWNGREEN,
        "berry" => Color::PURPLE,
        "wood" => Color::BROWN,
        _ => Color::GRAY,
    }
}

fn draw_world<D: RaylibDraw>(
    d: &mut D,
    state: &VillageState,
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

    let (villager_x, villager_y) = to_screen(state.villager.position);
    d.draw_circle(villager_x, villager_y, 3.0, Color::BLUE);

    for item in &state.items {
        let (item_x, item_y) = to_screen(item.position);
        d.draw_circle(item_x, item_y, 2.0, item_color(&item.id));
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

fn draw_hud<D: RaylibDraw>(d: &mut D) {
    d.draw_text("SCREEN AREA", 640, 10, 20, Color::RED);

    d.draw_rectangle(0, 0, SCREEN_WIDTH, 5, Color::RED);
    d.draw_rectangle(0, 5, 5, SCREEN_HEIGHT - 10, Color::RED);
    d.draw_rectangle(SCREEN_WIDTH - 5, 5, 5, SCREEN_HEIGHT - 10, Color::RED);
    d.draw_rectangle(0, SCREEN_HEIGHT - 5, SCREEN_WIDTH, 5, Color::RED);

    d.draw_rectangle(10, 10, 250, 113, Color::SKYBLUE.alpha(0.5));
    d.draw_rectangle_lines(10, 10, 250, 113, Color::BLUE);

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
}

pub fn run() {
    let (mut rl, thread) = init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Outbound")
        .build();
    rl.set_target_fps(60);

    let mut state = VillageState {
        villager: Villager::default(),
        items: spawn_items(&rl),
    };

    let villager_goals: Vec<Box<dyn Goal<VillageState>>> = vec![
        Box::new(CollectWood {}),
        Box::new(CollectStone {}),
        Box::new(CollectBerries {}),
    ];

    let mut plan_steps = next_plan(&state, &villager_goals);
    let mut walk: Vec<(i64, i64)> = vec![];
    let mut tick: u32 = 0;

    let building_site = Rectangle::new(30.0, 20.0, 25.0, 25.0);
    let villager_rect = Rectangle::new(10.0, 25.0, 10.0, 10.0);
    let mut player = Rectangle::new(400.0, 280.0, 40.0, 40.0);
    let buildings = spawn_skyline(&rl);

    let mut camera = Camera2D {
        target: Vector2::new(player.x + 20.0, player.y + 20.0),
        offset: Vector2::new(player.x, player.y),
        rotation: 0.0,
        zoom: 1.0,
    };

    while !rl.window_should_close() {
        if tick.is_multiple_of(TICKS_PER_ACTION) {
            advance(&mut state, &mut plan_steps, &mut walk, &villager_goals);
        }
        tick = tick.wrapping_add(1);

        handle_input(&rl, &mut player, &mut camera);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            draw_world(
                &mut d2,
                &state,
                &buildings,
                building_site,
                player,
                villager_rect,
            );
            draw_camera_axes(&mut d2, &camera);
        }
        draw_hud(&mut d);
    }
}
