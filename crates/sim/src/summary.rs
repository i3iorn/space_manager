//! A summary of a campaign, in plain text, for a playtester.
//!
//! **The report is what a playtester sends back.** It is plain text so it can
//! be pasted into an email, it names the seed so the campaign can be
//! reproduced exactly, and it says what happened: the money, the staff, the
//! programmes, the missions, and the worst of the chronicle.
//!
//! **It is a summary and not a transcript.** A thirty-year campaign has
//! thousands of chronicle entries, and a report that lists all of them is a
//! file nobody reads. What the reviewer needs is the shape of the campaign and
//! the moments that turned it.

use std::fmt::Write as _;

use domain::person::Role;
use domain::programme::ObjectiveKind;
use domain::research::{CAPABILITIES, DISCIPLINES};

use crate::Sim;

/// The summary of a campaign.
///
/// **Every number here is read out of the state**, so the report cannot
/// disagree with the campaign it describes.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn summary(sim: &Sim) -> String {
    let mut out = String::new();
    header(&mut out, sim);
    money(&mut out, sim);
    agency(&mut out, sim);
    programmes(&mut out, sim);
    missions(&mut out, sim);
    knowledge(&mut out, sim);
    chronicle(&mut out, sim);
    decisions(&mut out, sim);
    let _ = writeln!(out, "state hash {}", sim.state_hash());
    out
}

/// The header: what the campaign was, and how to reproduce it.
fn header(out: &mut String, sim: &Sim) {
    let state = sim.state();
    let _ = writeln!(out, "{} - {}", state.agency.name, sim.scenario().name);
    let _ = writeln!(out, "seed {}", sim.rng().root_seed());
    let now = sim.effective_now();
    let _ = writeln!(
        out,
        "date {} ({} months in)",
        sim.calendar().date(now),
        now.months()
    );
    out.push('\n');
}

/// The money.
fn money(out: &mut String, sim: &Sim) {
    let treasury = &sim.state().treasury;
    out.push_str("THE MONEY\n");
    let _ = writeln!(out, "  balance          {} credits", treasury.balance);
    let _ = writeln!(
        out,
        "  income a month   {} credits",
        treasury.total_income()
    );
    let _ = writeln!(
        out,
        "  outgoings a month {} credits",
        treasury.total_outgoings()
    );
    if !treasury.debt.is_empty() {
        let owed: u64 = treasury
            .debt
            .iter()
            .map(|debt| debt.outstanding.credits().unsigned_abs())
            .sum();
        let _ = writeln!(out, "  debt             {owed} credits");
    }
    let _ = writeln!(out, "  crisis           {}", treasury.crisis_stage.label());
    out.push('\n');
}

/// The agency: who works there, and how it stands.
fn agency(out: &mut String, sim: &Sim) {
    let state = sim.state();
    out.push_str("THE AGENCY\n");
    let _ = writeln!(out, "  staff            {}", state.people.len());
    for role in Role::ALL {
        let count = state
            .people
            .values()
            .filter(|person| person.role == role)
            .count();
        if count > 0 {
            let _ = writeln!(out, "    {:<18}{count}", role.label());
        }
    }
    let _ = writeln!(
        out,
        "  standing         goodwill {}%, prestige {}%, opinion {}%",
        state.political.goodwill.percent(),
        state.public.prestige.percent(),
        state.public.opinion.percent()
    );
    out.push('\n');
}

/// The programmes, and whether they were delivered.
fn programmes(out: &mut String, sim: &Sim) {
    let state = sim.state();
    out.push_str("THE PROGRAMMES\n");
    if state.programmes.is_empty() {
        out.push_str("  none\n");
    }
    for programme in state.programmes.values() {
        let _ = writeln!(out, "  {} [{}]", programme.title, programme.status.label());
        for objective in &programme.objectives {
            let kind = match objective.kind {
                ObjectiveKind::Primary => "primary",
                ObjectiveKind::Secondary => "secondary",
            };
            let met = match objective.met {
                Some(true) => "met",
                Some(false) => "missed",
                None => "open",
            };
            let _ = writeln!(out, "    [{kind}] {} - {met}", objective.description);
        }
    }
    out.push('\n');
}

/// The missions, and what became of them.
fn missions(out: &mut String, sim: &Sim) {
    let state = sim.state();
    out.push_str("THE MISSIONS\n");
    if state.missions.is_empty() {
        out.push_str("  none\n");
    }
    for mission in state.missions.values() {
        let outcome = match &mission.outcome {
            None => "still in preparation",
            Some(outcome) if outcome.success && !outcome.partial => "worked",
            Some(outcome) if outcome.partial => "partly worked",
            Some(_) => "lost",
        };
        let _ = writeln!(
            out,
            "  {} to {} - {outcome}",
            mission.destination,
            sim.calendar().date(mission.launch_window)
        );
    }
    out.push('\n');
}

/// What the agency knows.
fn knowledge(out: &mut String, sim: &Sim) {
    let state = sim.state();
    out.push_str("WHAT THE AGENCY KNOWS\n");
    let mut knew_anything = false;
    for discipline in &DISCIPLINES {
        let points = state.knowledge.get(&discipline.id).copied().unwrap_or(0);
        if points > 0 {
            let _ = writeln!(out, "  {:<24}{points} points", discipline.name);
            knew_anything = true;
        }
    }
    if !knew_anything {
        out.push_str("  nothing yet\n");
    }
    let unlocked: Vec<&str> = CAPABILITIES
        .iter()
        .filter(|capability| state.unlocked.contains(&capability.id))
        .map(|capability| capability.name)
        .collect();
    if unlocked.is_empty() {
        out.push_str("  no capability unlocked\n");
    } else {
        for name in unlocked {
            let _ = writeln!(out, "  {name}");
        }
    }
    out.push('\n');
}

/// The chronicle's worst moments.
fn chronicle(out: &mut String, sim: &Sim) {
    let state = sim.state();
    out.push_str("THE CHRONICLE\n");
    let mut notable: Vec<&domain::chronicle::ChronicleEntry> = state
        .chronicle
        .iter()
        .filter(|entry| entry.severity >= domain::chronicle::Severity::Major)
        .collect();
    notable.sort_by_key(|entry| entry.date);
    for entry in notable.iter().take(20) {
        let _ = writeln!(
            out,
            "  {} | {}",
            sim.calendar().date(entry.date),
            entry.headline
        );
    }
    if notable.len() > 20 {
        let _ = writeln!(out, "  ... and {} more", notable.len() - 20);
    }
    out.push('\n');
}

/// The decisions, because a campaign is its decisions.
fn decisions(out: &mut String, sim: &Sim) {
    out.push_str("THE DECISIONS\n");
    if sim.log().is_empty() {
        out.push_str("  none\n");
    }
    for entry in sim.log() {
        let _ = writeln!(
            out,
            "  {} | {}",
            sim.calendar().date(entry.issued_at),
            entry.kind
        );
    }
    out.push('\n');
}
