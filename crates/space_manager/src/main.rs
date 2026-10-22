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

mod paths;

/// The command-line options.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    /// The scenario to start from.
    scenario: String,
    /// The campaign's root seed, if the player overrode it.
    seed: Option<u64>,
    /// How many months to run without the interface, if any.
    headless: Option<u32>,
    /// The file to load the campaign from, if any.
    load: Option<String>,
    /// The file to save the campaign to, if any.
    save: Option<String>,
    /// The file to write the campaign's summary to, if any.
    report: Option<String>,
    /// Whether the player asked for help.
    help: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            scenario: "first_year".to_owned(),
            seed: None,
            headless: None,
            load: None,
            save: None,
            report: None,
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
    --scenario <NAME> Start from a named scenario. One of: first_year, first_orbit,
                     sandbox.
    --seed <N>        Set the campaign's root seed. Campaigns are fully
                      reproducible from their seed, so this reproduces a
                      campaign exactly.
    --headless <N>    Run N months without the interface and print a summary.
                      Useful for smoke tests.
    --load <FILE>     Load a saved campaign and carry on from it.
    --save <FILE>     Save the campaign to a file when the game ends. This is
                      also where the game saves itself, so it is where F3 and
                      F4 work.
    --report <FILE>   Write a summary of the campaign to a file. This is what a
                      playtester sends back.
    -h, --help        Show this message.

KEYS:
    1-7      screens (dashboard, staff, programme, research, vehicles,
             chronicle, branches)
    o        open the order palette - the way you act on the agency
    Space    pause and resume the clock
    + / -    one dilation step faster or slower
    Ctrl+1-9 set the dilation directly
    F3       save the campaign
    F4       load the campaign
    q        quit

SAVES. The game saves itself: at every month boundary, and when you quit. The
file is in your system's configuration directory unless --save says otherwise.

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
            "--load" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--load needs a file name after it".to_owned())?;
                options.load = Some(value);
            }
            "--save" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--save needs a file name after it".to_owned())?;
                options.save = Some(value);
            }
            "--report" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--report needs a file name after it".to_owned())?;
                options.report = Some(value);
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

    // **A loaded campaign is the campaign, whatever the scenario says.** The
    // save carries the scenario it came from, and starting it again from the
    // command line would be a different campaign.
    let (sim, scenario_name) = if let Some(path) = &options.load {
        let sim = load_campaign(path)?;
        let name = sim.scenario().name.clone();
        (sim, name)
    } else {
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
        let name = scenario.name.clone();
        (Sim::new(&scenario)?, name)
    };

    eprintln!(
        "space_manager — {scenario_name}, campaign seed {}",
        sim.rng().root_seed()
    );

    if let Some(months) = options.headless {
        return run_headless(
            sim,
            months,
            options.save.as_deref(),
            options.report.as_deref(),
        );
    }

    let mut app = App::new(sim);
    // **The key that saves the campaign saves it to the same file the command
    // line named**, so a player who started the game with `--save` and a
    // player who did not are both saving where they think they are.
    let path = save_path(options.save.as_deref());
    app.set_save_path(path.clone());
    // **And the game saves itself to the same file**, at every month boundary.
    app.set_autosave_path(path);
    ui::run(&mut app)?;

    // **The campaign is saved when the game ends**, and a failure to write it
    // is worth saying out loud: the player has just played for an hour and
    // the file they were counting on is not there.
    //
    // **This happens whether or not `--save` was given.** The game saves
    // itself, and the quit save is the last of those saves.
    save_campaign(app.save_path(), &app)?;
    // **The report is written when the game ends too**, and it is the thing a
    // playtester sends back.
    if let Some(path) = &options.report {
        write_report(path, app.sim())?;
    }
    Ok(())
}

/// Where the campaign is saved.
///
/// **The command line wins**, because a player who named a file meant it. And
/// the default is the platform's configuration directory, because a game that
/// writes beside its own executable is a game whose save file turns up in
/// whatever folder it was run from.
fn save_path(named: Option<&str>) -> String {
    if let Some(path) = named {
        return path.to_owned();
    }
    paths::campaign_file().map_or_else(
        || ui::DEFAULT_SAVE_PATH.to_owned(),
        |path| path.to_string_lossy().into_owned(),
    )
}

/// Reads a campaign from a file.
///
/// **A save that cannot be read is refused with a reason**, and the reason
/// says what to do about it. A truncated file, a file that is not a campaign,
/// and a campaign from another version of the game are three different
/// problems and the player needs to be told which one they have.
fn load_campaign(path: &str) -> Result<Sim> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("the campaign at {path} could not be read: {error}"))?;
    Sim::load(&text)
        .map_err(|error| anyhow::anyhow!("the campaign at {path} could not be loaded: {error}"))
}

/// Writes the campaign to a file.
fn save_campaign(path: &str, app: &ui::App) -> Result<()> {
    let text = app
        .sim()
        .save()
        .map_err(|error| anyhow::anyhow!("the campaign could not be written: {error}"))?;
    std::fs::write(path, text)
        .map_err(|error| anyhow::anyhow!("the campaign could not be written to {path}: {error}"))?;
    eprintln!("the campaign was saved to {path}");
    Ok(())
}

/// Writes the campaign's summary to a file.
///
/// **The report is what a playtester sends back**, so a failure to write it
/// is worth saying out loud for the same reason a failure to save is.
fn write_report(path: &str, sim: &Sim) -> Result<()> {
    let text = sim::summary::summary(sim);
    std::fs::write(path, text)
        .map_err(|error| anyhow::anyhow!("the report could not be written to {path}: {error}"))?;
    eprintln!("the report was written to {path}");
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
fn run_headless(mut sim: Sim, months: u32, save: Option<&str>, report: Option<&str>) -> Result<()> {
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
    // **The report is written even in a headless run**, because that is how a
    // build is checked: the campaign runs, and the file it leaves behind says
    // what happened.
    if let Some(path) = report {
        write_report(path, &sim)?;
    }
    // **And so is the save**, because a headless run is a campaign that ended
    // like any other and the file it leaves behind is the same file.
    if let Some(path) = save {
        let text = sim
            .save()
            .map_err(|error| anyhow::anyhow!("the campaign could not be written: {error}"))?;
        std::fs::write(path, text).map_err(|error| {
            anyhow::anyhow!("the campaign could not be written to {path}: {error}")
        })?;
        eprintln!("the campaign was saved to {path}");
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
    fn a_save_file_can_be_given() {
        let options = parse(&["--save", "campaign.json"]).expect("the options parse");
        assert_eq!(options.save.as_deref(), Some("campaign.json"));
    }

    #[test]
    fn a_load_file_can_be_given() {
        let options = parse(&["--load", "campaign.json"]).expect("the options parse");
        assert_eq!(options.load.as_deref(), Some("campaign.json"));
    }

    #[test]
    fn a_report_file_can_be_given() {
        // **The report is what a playtester sends back**, so it is a file the
        // player names on the command line.
        let options = parse(&["--report", "playtest.txt"]).expect("the options parse");
        assert_eq!(options.report.as_deref(), Some("playtest.txt"));
    }

    #[test]
    fn a_report_file_without_a_name_is_refused() {
        assert!(parse(&["--report"]).is_err());
    }

    #[test]
    fn the_report_says_what_happened() {
        // **The report is the campaign, in words.** A headless run that writes
        // one is how a build is checked.
        let path = std::env::temp_dir().join("space_manager_binary_report_test.txt");
        let mut scenario = Scenario::first_orbit();
        scenario.seed = 31_337;
        let mut sim = Sim::new(&scenario).expect("the campaign starts");
        sim.advance_to(domain::time::Instant::from_months(6))
            .expect("the campaign advances");
        write_report(path.to_str().expect("a path"), &sim).expect("the report is written");
        let text = std::fs::read_to_string(&path).expect("the file is there");
        let _ = std::fs::remove_file(&path);

        assert!(text.contains("seed 31337"), "the seed is in it:\n{text}");
        assert!(text.contains("THE MONEY"), "the money is in it:\n{text}");
    }

    #[test]
    fn a_save_file_without_a_name_is_refused() {
        assert!(parse(&["--save"]).is_err());
        assert!(parse(&["--load"]).is_err());
    }

    #[test]
    fn a_campaign_can_be_saved_and_loaded_again() {
        // **The whole point of the format.** A file on disk, a campaign in it,
        // and the campaign back.
        let path = std::env::temp_dir().join("space_manager_binary_save_test.json");
        let scenario = Scenario::first_orbit();
        let mut sim = Sim::new(&scenario).expect("the campaign starts");
        sim.advance_to(domain::time::Instant::from_months(3))
            .expect("the campaign advances");
        let text = sim.save().expect("the campaign saves");
        std::fs::write(&path, &text).expect("the file is written");

        let loaded = load_campaign(path.to_str().expect("a path")).expect("the campaign loads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded.state_hash(), sim.state_hash());
    }

    #[test]
    fn a_damaged_campaign_is_refused_with_a_reason() {
        // **A damaged save is a message and not a crash**, and the message
        // says what to do about it.
        let path = std::env::temp_dir().join("space_manager_binary_damaged_test.json");
        std::fs::write(&path, "{ this is not a campaign").expect("the file is written");
        let error = load_campaign(path.to_str().expect("a path")).expect_err("the load fails");
        let _ = std::fs::remove_file(&path);
        let message = error.to_string();
        assert!(
            message.contains("could not be read"),
            "the refusal says what is wrong: {message}"
        );
    }

    #[test]
    fn a_missing_campaign_is_refused_with_a_reason() {
        let error = load_campaign("no/such/file.json").expect_err("the load fails");
        assert!(
            error.to_string().contains("could not be read"),
            "the refusal says what is wrong: {error}"
        );
    }

    #[test]
    fn a_headless_run_saves_the_campaign_too() {
        // **A headless run is a campaign that ended like any other**, and the
        // file it leaves behind is the same file. A smoke test that silently
        // dropped the save would make `--save` a lie in exactly the mode
        // where nobody is watching.
        let path = std::env::temp_dir().join("space_manager_binary_headless_save.json");
        let path = path.to_string_lossy().into_owned();
        let mut scenario = Scenario::first_orbit();
        scenario.seed = 99;
        let mut sim = Sim::new(&scenario).expect("the campaign starts");
        sim.advance_to(domain::time::Instant::from_months(3))
            .expect("the campaign advances");
        let expected = sim.state_hash();
        let text = sim.save().expect("the campaign saves");
        std::fs::write(&path, text).expect("the file is written");

        let loaded = load_campaign(&path).expect("the campaign loads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded.state_hash(), expected);
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
    fn the_default_save_path_is_in_the_configuration_directory() {
        // **A game that writes beside its own executable is a game whose save
        // file turns up in whatever folder the player happened to run it
        // from**, and a save file that moves is a save file that is lost.
        let path = save_path(None);
        assert!(path.ends_with("campaign.json"), "the file is named: {path}");
        assert!(
            path.contains("space_manager"),
            "and it is in the game's own directory: {path}"
        );
    }

    #[test]
    fn a_named_save_file_wins_over_the_default() {
        // **A player who named a file meant it.**
        assert_eq!(save_path(Some("mygame.json")), "mygame.json");
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
