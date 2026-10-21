//! Writes the golden test's fixtures.
//!
//! Run with `cargo run -p sim --example make_fixture`. It writes:
//!
//! * `tests/fixtures/first_year/log.json` - the order log, serialised by
//!   serde so it cannot drift from the `OrderKind` type.
//! * `tests/fixtures/first_year/expected_state_hash.txt` — the campaign's state
//!   hash.
//! * `tests/fixtures/first_year/expected_chronicle.txt` — the campaign's
//!   history, as readable text.
//!
//! **Regenerating a fixture is a deliberate act.** The diff in the chronicle is
//! the review artefact: read it before committing it.

use domain::ids::{PersonId, ResearchProjectId};
use domain::money::Money;
use domain::order::OrderKind;
use domain::person::{Assignment, Role};
use domain::research::DisciplineId;
use sim::{Scenario, Sim};

/// The seed the golden campaign runs under.
const SEED: u64 = 6_125_084_923_715_548;

fn main() {
    let months = fixture_months();
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("first_year");
    std::fs::create_dir_all(&fixture_dir).expect("the fixture directory can be created");

    // The log.
    let json = serde_json::to_string_pretty(&months).expect("the orders serialise");
    write(&fixture_dir.join("log.json"), &format!("{json}\n"));

    // The seed.
    write(&fixture_dir.join("seed.txt"), &format!("{SEED}\n"));

    // The campaign, and its expected output.
    let mut scenario = Scenario::by_id("first_year").expect("the scenario exists");
    scenario.seed = SEED;

    let mut sim = Sim::new(&scenario).expect("the scenario starts");
    for month in &months {
        sim.step(month.clone()).expect("the month resolves");
    }

    let calendar = sim.calendar();
    let chronicle = sim
        .state()
        .chronicle
        .iter()
        .map(|entry| {
            format!(
                "{} | {} | {}",
                calendar.date(entry.date),
                entry.severity.label(),
                entry.headline
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    write(
        &fixture_dir.join("expected_state_hash.txt"),
        &format!("{}\n", sim.state_hash()),
    );
    write(
        &fixture_dir.join("expected_chronicle.txt"),
        &format!("{chronicle}\n"),
    );

    println!("wrote fixtures to {}", fixture_dir.display());
}

/// Writes a file, creating it if needed.
fn write(path: &std::path::Path, contents: &str) {
    std::fs::write(path, contents)
        .unwrap_or_else(|error| panic!("{} could not be written: {error}", path.display()));
}

/// The orders the golden campaign issues.
///
/// The campaign hires, starts two research projects, and staffs them. Note that
/// an order can only refer to something that already exists when it is
/// **validated**, so a project is created a month before anyone is assigned
/// to it.
fn fixture_months() -> Vec<Vec<OrderKind>> {
    let mut months: Vec<Vec<OrderKind>> = vec![
        // Month 0: hire a scientist and start the propulsion project.
        vec![
            OrderKind::Hire {
                role: Role::Scientist,
                salary: Money::from_credits(1_200),
            },
            OrderKind::StartProject {
                discipline: DisciplineId(0),
                principal_investigator: PersonId(0),
                title: "Liquid upper stage".to_owned(),
            },
        ],
        // Month 1: put two more people on the propulsion project.
        vec![
            OrderKind::Assign {
                person: PersonId(1),
                to: Assignment::Project {
                    project: ResearchProjectId(1),
                },
            },
            OrderKind::Assign {
                person: PersonId(3),
                to: Assignment::Project {
                    project: ResearchProjectId(1),
                },
            },
        ],
        // Month 2: start the life support project.
        vec![OrderKind::StartProject {
            discipline: DisciplineId(2),
            principal_investigator: PersonId(2),
            title: "Closed-loop systems".to_owned(),
        }],
        // Month 3: staff the life support project.
        vec![OrderKind::Assign {
            person: PersonId(2),
            to: Assignment::Project {
                project: ResearchProjectId(2),
            },
        }],
    ];

    // The rest of the first three years.
    while months.len() < 36 {
        months.push(Vec::new());
    }
    months
}
