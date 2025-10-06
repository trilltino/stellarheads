use stellar_heads_game::create_app;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    println!("🎮 Starting Stellar Heads native client...");

    create_app().run();
}