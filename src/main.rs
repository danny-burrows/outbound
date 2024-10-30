use outbound::Villager;

fn main() {
    let villager = Villager::new();
    let villager_is_alive = villager.is_alive();

    println!("Hello, world! Villager is alive: {villager_is_alive}");
}
