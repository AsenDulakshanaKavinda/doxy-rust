


fn main() {
    let cfg = config::Config::load().expect("Failed to load application configuration");
    println!("Server starting on port {}", cfg.app_port);
}