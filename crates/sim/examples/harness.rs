//! The balance harness, run from the command line.
//!
//! ```text
//! cargo run -p sim --example harness -- [scenario] [seeds] [months]
//! ```
//!
//! **The report is the deliverable of a balance pass.** It says how often each
//! reference policy reached orbit, what it cost, and what it left behind — and
//! a balance change that ruins one of the policies shows up here before it
//! reaches a playtester.

use sim::Scenario;
use sim::harness;

fn main() {
    let mut args = std::env::args().skip(1);
    let scenario_id = args.next().unwrap_or_else(|| "first_orbit".to_owned());
    let seeds = args
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(20);
    let months = args
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(60);

    let scenario = Scenario::by_id(&scenario_id).unwrap_or_else(|| {
        eprintln!(
            "there is no scenario called {scenario_id:?}. The scenarios are: {}",
            Scenario::ids().join(", ")
        );
        std::process::exit(1);
    });

    println!("scenario: {} ({})", scenario.name, scenario.id);
    println!("seeds: {seeds}, months: {months}");
    println!();

    let outcomes = harness::run_all(&scenario, &harness::default_seeds(seeds), months);
    println!("{}", harness::report(&outcomes));

    if !harness::all_viable(&outcomes) {
        println!("**A policy has collapsed. The trade-offs are not real.**");
        std::process::exit(2);
    }
}
