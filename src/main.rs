use outbound::goap::{generate_goal_state, plan, print_plan, Action, Agent, Item, State};
use outbound::Villager;
use raylib::consts::KeyboardKey::*;
use raylib::prelude::*;

const MAX_BUILDINGS: usize = 100;
const MAX_TREES: usize = 10;
const MAX_BERRIES: usize = 4;
const MAX_STONE: usize = 5;

fn main() {
    let villager = Villager::new();
    let villager_is_alive = villager.is_alive();
    let villager_rect = Rectangle::new(10.0, 25.0, 10.0, 10.0);

    println!("Hello, world! Villager is alive: {villager_is_alive}");

    let (w, h) = (860, 640);
    let (mut rl, thread) = raylib::init().size(w, h).title("Outbound").build();
    rl.set_target_fps(60);

    let agent = Agent::new((0, 0));

    let mut items = Vec::with_capacity(MAX_BERRIES + MAX_STONE + MAX_TREES);
    for _ in 0..MAX_TREES {
        let rx: i64 = rl.get_random_value(-10..10);
        let ry: i64 = rl.get_random_value(-10..10);
        items.push(Item::new("tree".to_string(), (rx, ry)));
    }
    for _ in 0..MAX_BERRIES {
        let rx: i64 = rl.get_random_value(-10..10);
        let ry: i64 = rl.get_random_value(-10..10);
        items.push(Item::new("berry".to_string(), (rx, ry)));
    }
    for _ in 0..MAX_STONE {
        let rx: i64 = rl.get_random_value(-10..10);
        let ry: i64 = rl.get_random_value(-10..10);
        items.push(Item::new("stone".to_string(), (rx, ry)));
    }

    let current_state = State::construct(&agent, &items);
    let goal_state = generate_goal_state(&current_state);

    // let available_actions = generate_available_actions(agent, items);
    let plan_option = plan(current_state.clone(), goal_state);

    let mut state = current_state.clone();
    let plan = plan_option.expect("FAILED TO PLAN");

    print_plan(plan.clone());

    let mut plan_iter = plan.iter();
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

    let mut act_offset = 0;

    while !rl.window_should_close() {
        if act_offset % 90 == 0 {
            if let Some(action) = plan_iter.next() {
                state = action.act(state);
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

            for i in 0..MAX_BUILDINGS {
                d2.draw_rectangle_rec(buildings[i], build_colors[i]);
            }

            d2.draw_circle(
                state.agent.position.0 as i32,
                state.agent.position.1 as i32,
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
