use outbound::goap::{generate_goal_state, plan, print_plan, Action, Agent, Item, State};
use outbound::Villager;
use std::{thread, time};

fn print_world(agent: &Agent, items: &[Item]) {
    let agent_position = agent.position;

    for i in 0..10 {
        for j in 0..10 {
            if let Some(item) = items.iter().find(|item| item.position == (i, j)) {
                print!("{}", item.id.chars().next().unwrap());
                continue;
            }

            if (i, j) == agent_position {
                print!("X");
                continue;
            }

            print!("-");
        }
        println!();
    }
}

fn main() {
    let villager = Villager::new();
    let villager_is_alive = villager.is_alive();

    println!("Hello, world! Villager is alive: {villager_is_alive}");

    let agent = Agent::new((0, 0));

    let items = vec![
        Item::new("tree".to_string(), (1, 5)),
        Item::new("tree".to_string(), (3, 2)),
        Item::new("tree".to_string(), (2, 2)),
        Item::new("tree".to_string(), (4, 3)),
        Item::new("stone".to_string(), (1, 2)),
        Item::new("berry".to_string(), (4, 4)),
    ];

    let current_state = State::construct(&agent, &items);
    let goal_state = generate_goal_state(&current_state);

    // let available_actions = generate_available_actions(agent, items);
    let plan_option = plan(current_state.clone(), goal_state);

    let sleep_ms = time::Duration::from_millis(500);

    if let Some(plan) = plan_option {
        print_plan(plan.clone());
        let mut state = current_state.clone();

        thread::sleep(sleep_ms);

        print!("{}[2J", 27 as char);
        print_world(&state.agent, &state.items);
        for action in plan.into_iter() {
            state = action.act(state);
            thread::sleep(sleep_ms);
            print!("{}[2J", 27 as char);
            print_world(&state.agent, &state.items);
        }
    } else {
        println!("Plan failed!");
    }
}
