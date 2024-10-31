use outbound::goap::{
    generate_available_actions, generate_goal_state, plan, print_plan, Agent, Item, State,
};
use outbound::Villager;

fn main() {
    let villager = Villager::new();
    let villager_is_alive = villager.is_alive();

    println!("Hello, world! Villager is alive: {villager_is_alive}");

    let agent = Agent::new((0, 0));

    let items = vec![
        Item::new("wood".to_string(), (5, 5)),
        Item::new("wood".to_string(), (1, 1)),
        Item::new("wood".to_string(), (2, 2)),
        Item::new("wood".to_string(), (3, 3)),
        Item::new("stone".to_string(), (1, 2)),
        Item::new("berry".to_string(), (4, 4)),
    ];

    let current_state = State::construct(&agent, &items);
    let goal_state = generate_goal_state(&current_state);

    let available_actions = generate_available_actions(agent, items);
    let plan_option = plan(current_state, available_actions, goal_state);

    if let Some(plan) = plan_option {
        print_plan(plan);
    } else {
        println!("Plan failed!");
    }
}
