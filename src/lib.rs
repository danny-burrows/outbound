mod actions;
mod goap;
mod item;
mod villager;

use crate::actions::VillageState;
use crate::goap::Action;
use crate::goap::{plan, print_plan};
use crate::item::Item;
use crate::villager::Villager;
use actions::VillagerActionEnum;
use raylib::consts::KeyboardKey::*;
use raylib::prelude::*;

const MAX_BUILDINGS: usize = 100;
const MAX_TREES: usize = 250;
const MAX_BERRIES: usize = 50;
const MAX_STONE: usize = 25;

pub(crate) fn generate_goal_state(current_state: &VillageState) -> VillageState {
    let mut goal_state = current_state.clone();
    goal_state.villager.inventory.push("wood".to_string());
    goal_state.villager.inventory.push("stone".to_string());
    goal_state.villager.inventory.push("berry".to_string());
    goal_state.villager.inventory.push("wood".to_string());
    goal_state.villager.inventory.push("wood".to_string());
    goal_state
}

pub fn run() {
    let villager = Villager::default();
    let villager_is_alive = villager.is_alive();
    let villager_rect = Rectangle::new(10.0, 25.0, 10.0, 10.0);

    println!("Hello, world! Villager is alive: {villager_is_alive}");

    let (w, h) = (860, 640);
    let (mut rl, thread) = raylib::init().size(w, h).title("Outbound").build();
    rl.set_target_fps(60);

    let mut items = Vec::with_capacity(MAX_BERRIES + MAX_STONE + MAX_TREES);
    for _ in 0..MAX_TREES {
        let rx: i64 = rl.get_random_value(0..150);
        let ry: i64 = rl.get_random_value(0..150);
        items.push(Item::new("tree".to_string(), (rx, ry)));
    }
    for _ in 0..MAX_BERRIES {
        let rx: i64 = rl.get_random_value(0..150);
        let ry: i64 = rl.get_random_value(0..150);
        items.push(Item::new("berry".to_string(), (rx, ry)));
    }
    for _ in 0..MAX_STONE {
        let rx: i64 = rl.get_random_value(0..150);
        let ry: i64 = rl.get_random_value(0..150);
        items.push(Item::new("stone".to_string(), (rx, ry)));
    }

    let current_state = VillageState { villager, items };
    let goal_state = generate_goal_state(&current_state);

    // let available_actions = generate_available_actions(agent, items);
    let plan_option = plan(current_state.clone(), goal_state);

    let mut state = current_state.clone();
    let plan: Vec<VillagerActionEnum> = plan_option.expect("FAILED TO PLAN");

    print_plan(plan.clone());

    let mut plan_iter = plan.iter();

    let building_site = Rectangle::new(30.0, 20.0, 25.0, 25.0);

    let mut player = Rectangle::new(400.0, 280.0, 40.0, 40.0);
    let mut buildings = Vec::with_capacity(MAX_BUILDINGS);
    let mut build_colors = Vec::with_capacity(MAX_BUILDINGS);
    let mut spacing = 0.0;
    for i in 0..MAX_BUILDINGS {
        let bh: i32 = rl.get_random_value(100..800);
        buildings.push(Rectangle::new(
            -6000.0 + spacing,
            (h - 130 - bh) as f32,
            rl.get_random_value::<i32>(50..200) as f32,
            bh as f32,
        ));

        spacing += buildings[i].width;
        build_colors.push(Color::new(
            rl.get_random_value::<i32>(200..240) as u8,
            rl.get_random_value::<i32>(200..240) as u8,
            rl.get_random_value::<i32>(200..240) as u8,
            255,
        ));
    }

    let mut camera = Camera2D {
        target: Vector2::new(player.x + 20.0, player.y + 20.0),
        offset: Vector2::new(player.x, player.y),
        rotation: 0.0,
        zoom: 1.0,
    };

    let mut movement_left = vec![];
    let mut act_offset = 0;

    while !rl.window_should_close() {
        if act_offset % 20 == 0 {
            if let Some(p) = movement_left.pop() {
                state.villager.position = p;
            } else if let Some(current_action) = plan_iter.next() {
                if let crate::actions::VillagerActionEnum::MoveToNearestItemAction(move_action) =
                    current_action
                {
                    let goal_state = move_action.act(state.clone());
                    let (gx, gy) = state.villager.position;

                    if let Some((movement, _)) = pathfinding::directed::astar::astar(
                        &goal_state.villager.position,
                        |(x, y)| -> Vec<((i64, i64), i32)> {
                            (-1..=1)
                                .map(move |i| (-1..=1).map(move |j| ((x + i, y + j), 1)))
                                .flatten()
                                .collect()
                        },
                        |&(x, y)| -> i32 {
                            (((gx - x) as f32).powf(2.0) + ((gy - y) as f32).powf(2.0)).sqrt()
                                as i32
                        },
                        |&(x, y)| -> bool { x == gx && y == gy },
                    ) {
                        movement_left = movement;
                    }
                } else {
                    state = current_action.act(state);
                }
            }
        }
        act_offset += 1;

        if rl.is_key_down(KEY_RIGHT) {
            player.x += 2.0;
        } else if rl.is_key_down(KEY_LEFT) {
            player.x -= 2.0;
        }

        // Camera follows player
        // camera.target = Vector2::new(
        //     player.x + player.width / 2.0,
        //     player.y + player.height / 2.0,
        // );
        camera.target = Vector2::new(0.0, 0.0);

        // Camera rotation controls
        if rl.is_key_down(KEY_A) {
            camera.rotation -= 1.0;
        } else if rl.is_key_down(KEY_S) {
            camera.rotation += 1.0;
        }

        // Limit camera rotation to 80 degrees
        camera.rotation = camera.rotation.max(-40.0).min(40.0);

        // zoom controls
        camera.zoom += rl.get_mouse_wheel_move() * 0.05;
        camera.zoom = camera.zoom.max(0.1).min(3.0);

        if rl.is_key_pressed(KEY_R) {
            camera.zoom = 1.0;
            camera.rotation = 0.0;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            d2.draw_rectangle(-6000, 320, 13000, 8000, Color::DARKGRAY);

            d2.draw_rectangle_lines_ex(building_site, 1.0, Color::LIMEGREEN);

            for i in 0..MAX_BUILDINGS {
                d2.draw_rectangle_rec(buildings[i], build_colors[i]);
            }

            d2.draw_circle(
                state.villager.position.0 as i32,
                state.villager.position.1 as i32,
                3.0,
                Color::BLUE,
            );

            for i in state.items.clone().into_iter() {
                let c = if &i.id == "tree" {
                    Color::LAWNGREEN
                } else if &i.id == "berry" {
                    Color::PURPLE
                } else if &i.id == "wood" {
                    Color::BROWN
                } else {
                    Color::GRAY
                };
                d2.draw_circle(i.position.0 as i32, i.position.1 as i32, 2.0, c);
            }
            d2.draw_rectangle_rec(villager_rect, Color::BLUE);
            d2.draw_rectangle_rec(player, Color::RED);
            d2.draw_line(
                camera.target.x as i32,
                -h * 10,
                camera.target.x as i32,
                h * 10,
                Color::GREEN,
            );
            d2.draw_line(
                -w * 10,
                camera.target.y as i32,
                w * 10,
                camera.target.y as i32,
                Color::GREEN,
            );
        }

        d.draw_text("SCREEN AREA", 640, 10, 20, Color::RED);

        d.draw_rectangle(0, 0, w, 5, Color::RED);
        d.draw_rectangle(0, 5, 5, h - 10, Color::RED);
        d.draw_rectangle(w - 5, 5, 5, h - 10, Color::RED);
        d.draw_rectangle(0, h - 5, w, 5, Color::RED);

        d.draw_rectangle(10, 10, 250, 113, Color::SKYBLUE.fade(0.5));
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
}
