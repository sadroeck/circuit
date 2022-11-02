use circuits::{thread, Proc, ProcExt};
use std::time::Duration;

fn main() {
    thread(print_ping_on_interval)
        .and_then(thread(print_pong_on_interval))
        .join()
        .expect("Could not join")
}

fn print_ping_on_interval() -> anyhow::Result<()> {
    for _ in 0..5 {
        std::thread::sleep(Duration::from_secs(1));
        println!("Ping");
    }
    Ok(())
}

fn print_pong_on_interval() -> anyhow::Result<()> {
    for _ in 0..5 {
        std::thread::sleep(Duration::from_secs(1));
        println!("Pong");
    }
    Ok(())
}
