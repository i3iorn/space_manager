//! A scaling benchmark.
//!
//! **The game is meant to reach three hundred people**, and the shipped
//! scenarios have twelve. This benchmark builds an agency of a given size and
//! says how long a five-year campaign takes, so that the cost of a change is a
//! number rather than a surprise.
//!
//! Run it with `cargo run -p sim --example scaling`.

use std::time::Instant;

use domain::time::Instant as GameInstant;
use sim::{Scenario, Sim};

fn main() {
    println!(
        "{:>7} {:>7} {:>7} {:>10} {:>10} {:>10} {:>8} {:>10} {:>10}",
        "staff",
        "projects",
        "months",
        "campaign",
        "per month",
        "per hour",
        "entries",
        "save",
        "fork"
    );
    for staff in [12, 50, 150, 300] {
        let projects = (staff / 5).max(1);
        let facilities = (staff / 30).max(1);
        let scenario = Scenario::scaling(staff, projects, facilities);
        let mut sim = Sim::new(&scenario).expect("the campaign starts");
        let started = Instant::now();
        sim.advance_to(GameInstant::from_months(60))
            .expect("the campaign advances");
        let elapsed = started.elapsed();
        let months = 60;
        let hours = u128::from(months * 30 * 24);

        // **The save, and the fork.** Both are what the interface asks for on
        // demand, and both are a whole campaign's worth of work.
        let save_started = Instant::now();
        let text = sim.save().expect("the campaign saves");
        let save = save_started.elapsed();
        let fork_started = Instant::now();
        let _branch = sim
            .fork(GameInstant::from_months(30))
            .expect("the campaign forks");
        let fork = fork_started.elapsed();

        println!(
            "{staff:>7} {projects:>7} {months:>7} {:>10} {:>10} {:>10} {:>8} {:>10} {:>10}",
            format!("{elapsed:.2?}"),
            format!("{:?}", elapsed / months),
            format!("{:?}", elapsed / hours as u32),
            sim.state().chronicle.len(),
            format!("{save:.2?}"),
            format!("{fork:.2?}"),
        );
        println!("      the save is {} bytes", text.len());
    }
}
