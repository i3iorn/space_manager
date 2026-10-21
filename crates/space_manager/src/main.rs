//! `space_manager` — a space agency management simulation.
//!
//! The binary is deliberately thin. It parses the command line, builds the
//! simulation, and hands off to the interface. **All game logic lives in the
//! `sim` crate and the domain model lives in the `domain` crate**; this file
//! contains no rules about the game.
//!
//! See `docs/ARCHITECTURE.md` §4 for the crate layout.

use std::process::ExitCode;

use anyhow::{Result, bail};
use sim::{Scenario, Sim};
use ui::App;

/// The command-line options.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    /// The scenario to start from.
    scenario: String,
    /// The campaign's root seed, if the player overrode it.
    seed: Option<u64>,
    /// How many months to run without the interface, if any.
    headless: Option<u32>,
    /// Whether the player asked for help.
    help: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            scenario: "first_year".to_owned(),
            seed: None,
            headless: None,
            help: false,
        }
    }
}

/// The text printed by `--help`.
const HELP: &str = "\
space_manager — a space agency management simulation

USAGE:
    space_manager [OPTIONS]

OPTIONS:
    --scenario <NAME> Start from a named scenario. One of: first_year, sandbox.
    --seed <N>        Set the campaign's root seed. Campaigns are fully
                      reproducible from their seed, so this reproduces a
                      campaign exactly.
    --headless <N>    Run N months without the interface and print a summary.
                      Useful for smoke tests.
    -h, --help        Show this message.

KEYS:
    1-7      screens (dashboard, staff, programme, research, vehicles,
             chronicle, branches)
    o        open the order palette - the way you act on the agency
    Space    pause and resume the clock
    + / -    one dilation step faster or slower
    Ctrl+1-9 set the dilation directly
    q        quit

THE CLOCK RUNS. The campaign advances whether or not you are touching
anything, and there is no month to end - press Space to stop and think.
";

/// Parses the command line.
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--seed needs a number after it".to_owned())?;
                options.seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("{value:?} is not a number"))?,
                );
            }
            "--scenario" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--scenario needs a name after it".to_owned())?;
                options.scenario = value;
            }
            "--headless" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--headless needs a number of months after it".to_owned())?;
                options.headless = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| format!("{value:?} is not a number of months"))?,
                );
            }
            "-h" | "--help" => options.help = true,
            _ => return Err(format!("unknown option {arg:?}")),
        }
    }

    Ok(options)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("space_manager: {error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Starts the game.
fn run() -> Result<()> {
    let options = parse_args(std::env::args().skip(1)).map_err(anyhow::Error::msg)?;

    if options.help {
        print!("{HELP}");
        return Ok(());
    }

    let Some(mut scenario) = Scenario::by_id(&options.scenario) else {
        bail!(
            "there is no scenario called {:?}. The scenarios are: {}",
            options.scenario,
            Scenario::ids().join(", ")
        );
    };

    if let Some(seed) = options.seed {
        scenario.seed = seed;
    }

    let sim = Sim::new(&scenario)?;
    eprintln!(
        "space_manager — {} ({}), campaign seed {}",
        scenario.name,
        scenario.id,
        sim.rng().root_seed()
    );

    if let Some(months) = options.headless {
        return run_headless(sim, months);
    }

    ui::run(App::new(sim))?;
    Ok(())
}

/// Runs the campaign without the interface and prints a summary.
///
/// This is the smoke test. It exists so that the game can be run without a
/// terminal — in CI, in a script, or by an automated session — and so that
/// "does it still start?" is a question with a cheap answer.
///
/// **It goes through `advance_to`**, which is the same path the interface's
/// clock uses, so the smoke test exercises the continuous model and not a
/// shortcut.
fn run_headless(mut sim: Sim, months: u32) -> Result<()> {
    let target = sim
        .now()
        .saturating_add(domain::time::Duration::from_months(u64::from(months)));
    sim.advance_to(target)?;

    let state = sim.state();
    println!(
        "date {} | staff {} | programmes {} | projects {} | facilities {} | chronicle {} | treasury {}",
        sim.calendar().date(sim.effective_now()),
        state.people.len(),
        state.programmes.len(),
        state.projects.len(),
        state.facilities.len(),
        state.chronicle.len(),
        state.treasury.balance,
    );
    // The work, so the smoke test shows the agency is *doing* something and not
    // merely standing there.
    for project in state.projects.values() {
        println!(
            "  project {}: {}/{} points, team {}",
            project.title,
            project.progress.whole_points(),
            project.progress.target / 256,
            project.team.len()
        );
    }
    for entry in state.chronicle.iter().rev().take(5) {
        println!("  {} {}", sim.calendar().date(entry.date), entry.headline);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Options, String> {
        parse_args(args.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn a_seed_can_be_given() {
        let options = parse(&["--seed", "42"]).expect("the options parse");
        assert_eq!(options.seed, Some(42));
    }

    #[test]
    fn a_scenario_can_be_given() {
        let options = parse(&["--scenario", "sandbox"]).expect("the options parse");
        assert_eq!(options.scenario, "sandbox");
    }

    #[test]
    fn the_default_scenario_is_the_first_year() {
        let options = parse(&[]).expect("the options parse");
        assert_eq!(options.scenario, "first_year");
    }

    #[test]
    fn an_unknown_option_is_refused() {
        assert!(parse(&["--nonsense"]).is_err());
    }

    #[test]
    fn a_seed_without_a_value_is_refused() {
        assert!(parse(&["--seed"]).is_err());
    }

    #[test]
    fn a_non_numeric_seed_is_refused() {
        assert!(parse(&["--seed", "later"]).is_err());
    }

    #[test]
    fn the_shipped_scenarios_all_start() {
        for id in Scenario::ids() {
            let scenario = Scenario::by_id(id).expect("the scenario exists");
            let sim = Sim::new(&scenario).expect("the scenario starts");
            assert_eq!(sim.now(), domain::time::Instant::ZERO);
        }
    }

    #[test]
    fn an_unknown_scenario_is_refused() {
        assert!(Scenario::by_id("atlantis").is_none());
    }

    #[test]
    fn the_headless_run_is_the_continuous_model() {
        // **The smoke test exercises the continuous model and not a shortcut**,
        // so a headless run and an interactive run at the same dilation are the
        // same campaign.
        let scenario = Scenario::first_year();
        let mut headless = Sim::new(&scenario).expect("the campaign starts");
        let target = domain::time::Instant::from_months(6);
        headless.advance_to(target).expect("advances");

        let mut interactive = Sim::new(&scenario).expect("the campaign starts");
        // The same span, in much smaller pieces.
        while interactive.now() < target {
            let next = interactive
                .now()
                .saturating_add(domain::time::Duration::from_hours(7))
                .min(target);
            interactive.advance_to(next).expect("advances");
        }

        assert_eq!(headless.now(), interactive.now());
        assert_eq!(headless.state_hash(), interactive.state_hash());
    }

    #[test]
    fn the_campaign_starts_with_something_to_do() {
        // **A small agency with a budget that will not last, three scientists,
        // and a research project already running.** The player's first decision
        // is about a project that exists.
        let scenario = Scenario::first_year();
        assert_eq!(scenario.start_state.people.len(), 3);
        assert_eq!(scenario.start_state.projects.len(), 1);
        assert_eq!(scenario.start_state.facilities.len(), 1);
    }
}
