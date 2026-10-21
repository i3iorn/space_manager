//! The event catalogue.
//!
//! **An event is data.** It is a weight, a set of conditions, a set of
//! effects, and the words it uses — and nothing more. Nothing in the
//! catalogue can break an invariant, because the catalogue cannot express an
//! arbitrary mutation.
//!
//! The design rule the whole thing rests on: **bad conditions raise the odds
//! of bad events, and good conditions raise the odds of good ones.** The player
//! should be able to look back at a disaster and say *my fault — I was running
//! everyone at eighty percent fatigue and skipped testing*.
//!
//! **Every weighting context corresponds to something the player did.** That
//! is the fairness rule, and it is checkable: if a bad event fires without a
//! matching context, the weighting is wrong. The eight contexts are in
//! `BALANCE.md` §8.2, and the catalogue covers all eight.

use domain::chronicle::{Category, Severity};
use domain::event::{Condition, Effect, EventDef, FollowUp, NarrativeSpec, WeightMod};
use domain::ids::{EventId, FacilityId, PersonId};
use domain::money::Money;

use std::sync::LazyLock;

/// The wildcard id a condition uses when it means *any*.
///
/// **The catalogue cannot name a person or a facility**, because the people in
/// the catalogue do not exist yet: a condition is checked against the state at
/// the moment the event fires.
pub const ANY_PERSON: PersonId = PersonId(u64::MAX);

/// The wildcard id a condition uses when it means *any* facility.
pub const ANY_FACILITY: FacilityId = FacilityId(u64::MAX);

/// A weight modifier that applies when the agency is worn down.
const TIRED: WeightMod = WeightMod {
    when: Condition::AverageFatigueAbove(70),
    percent: 300,
};

/// A weight modifier that applies when the equipment is worn.
const NEGLECTED: WeightMod = WeightMod {
    when: Condition::FacilityConditionBelow(ANY_FACILITY, 60),
    percent: 300,
};

/// A weight modifier that applies when the money is short.
const BROKE: WeightMod = WeightMod {
    when: Condition::RunwayBelowMonths(2),
    percent: 300,
};

/// A weight modifier that applies when the public has turned against the agency.
const UNPOPULAR: WeightMod = WeightMod {
    when: Condition::PublicOpinionBelow(-30),
    percent: 300,
};

/// A weight modifier that applies when a programme has no slack left.
const NO_SLACK: WeightMod = WeightMod {
    when: Condition::ProgrammeWithoutSlack,
    percent: 250,
};

/// A weight modifier that applies when something falls due soon.
const DUE_SOON: WeightMod = WeightMod {
    when: Condition::MilestoneDueWithinMonths(1),
    percent: 250,
};

/// A weight modifier that applies when a launch is risky.
pub const RISKY: WeightMod = WeightMod {
    when: Condition::LaunchRiskAbove(20),
    percent: 400,
};

/// A weight modifier that applies when nobody is working in a facility.
const UNSTAFFED: WeightMod = WeightMod {
    when: Condition::FacilityUnstaffed,
    percent: 300,
};

/// Every event the game can fire.
///
/// **The catalogue is data and it is built once.** It is a lazy static
/// because an event holds vectors, and a vector cannot be a constant.
static CATALOGUE: LazyLock<Vec<EventDef>> = LazyLock::new(|| {
    vec![
        // ------------------------------------------------------------- technical
        EventDef {
            id: EventId(0),
            category: Category::Technical,
            base_weight: 200,
            preconditions: vec![],
            weight_modifiers: vec![NEGLECTED, UNSTAFFED],
            effects: vec![Effect::AdjustAllCondition { delta: -5 }],
            narrative: NarrativeSpec {
                headline: "Equipment failed in a laboratory.".to_owned(),
                body: Some("A component that should have been replaced was not.".to_owned()),
                severity: Severity::Major,
                category: Category::Technical,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(1),
            category: Category::Technical,
            base_weight: 150,
            preconditions: vec![],
            weight_modifiers: vec![NEGLECTED],
            effects: vec![Effect::AdjustAllCondition { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "A calibration was found to be wrong.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Technical,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(2),
            category: Category::Technical,
            base_weight: 120,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllCondition { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A test rig gave a reading nobody trusted.".to_owned(),
                body: Some("It was recalibrated, and the morning was lost.".to_owned()),
                severity: Severity::Minor,
                category: Category::Technical,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(3),
            category: Category::Technical,
            base_weight: 100,
            preconditions: vec![],
            weight_modifiers: vec![RISKY],
            effects: vec![Effect::AdjustAllCondition { delta: -3 }],
            narrative: NarrativeSpec {
                headline: "A launch was scrubbed at the last minute.".to_owned(),
                body: Some("A fault was found in something that had flown before.".to_owned()),
                severity: Severity::Major,
                category: Category::Technical,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(4),
            category: Category::Technical,
            base_weight: 80,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPublicOpinion { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A component was recalled.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Technical,
            },
            follow_ups: vec![],
        },
        // ------------------------------------------------------------- personnel
        EventDef {
            id: EventId(5),
            category: Category::Personnel,
            base_weight: 180,
            preconditions: vec![],
            weight_modifiers: vec![TIRED],
            effects: vec![
                Effect::AdjustAllMorale { delta: -4 },
                Effect::AdjustAllLoyalty { delta: -2 },
            ],
            narrative: NarrativeSpec {
                headline: "A long shift went badly.".to_owned(),
                body: Some("Nobody said much afterwards.".to_owned()),
                severity: Severity::Major,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(6),
            category: Category::Personnel,
            base_weight: 140,
            preconditions: vec![],
            weight_modifiers: vec![TIRED],
            effects: vec![Effect::AdjustAllFatigue { delta: 4 }],
            narrative: NarrativeSpec {
                headline: "Somebody worked through the weekend.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(7),
            category: Category::Personnel,
            base_weight: 120,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllMorale { delta: 3 }],
            narrative: NarrativeSpec {
                headline: "A quiet week, and a good one.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(8),
            category: Category::Personnel,
            base_weight: 100,
            preconditions: vec![],
            weight_modifiers: vec![TIRED],
            effects: vec![Effect::AdjustAllHealth { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "Somebody was taken ill.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(9),
            category: Category::Personnel,
            base_weight: 90,
            preconditions: vec![],
            weight_modifiers: vec![UNSTAFFED],
            effects: vec![Effect::AdjustAllMorale { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "A laboratory ran itself, and did not do it well.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(10),
            category: Category::Personnel,
            base_weight: 70,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllLoyalty { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "The agency was told it was doing well.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        // ------------------------------------------------------------- discovery
        EventDef {
            id: EventId(11),
            category: Category::Discovery,
            base_weight: 90,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPublicOpinion { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "A result nobody expected.".to_owned(),
                body: Some("The data was better than the paper had claimed.".to_owned()),
                severity: Severity::Info,
                category: Category::Discovery,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(12),
            category: Category::Discovery,
            base_weight: 70,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPrestige { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "A paper was published, and it was read.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Discovery,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(13),
            category: Category::Discovery,
            base_weight: 60,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllMorale { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "Somebody's life's work turned out to be right.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Discovery,
            },
            follow_ups: vec![],
        },
        // ------------------------------------------------------------- political
        EventDef {
            id: EventId(14),
            category: Category::Political,
            base_weight: 160,
            preconditions: vec![],
            weight_modifiers: vec![BROKE, NO_SLACK, DUE_SOON],
            effects: vec![Effect::AdjustApproval { delta: -3 }],
            narrative: NarrativeSpec {
                headline: "A review of the agency's work was requested.".to_owned(),
                body: Some("The state would like to see the numbers.".to_owned()),
                severity: Severity::Major,
                category: Category::Political,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(15),
            category: Category::Political,
            base_weight: 120,
            preconditions: vec![],
            weight_modifiers: vec![UNPOPULAR],
            effects: vec![Effect::AdjustApproval { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "A sponsor asked awkward questions.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Political,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(16),
            category: Category::Political,
            base_weight: 110,
            preconditions: vec![],
            weight_modifiers: vec![NO_SLACK],
            effects: vec![Effect::AdjustApproval { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "A programme was noted as behind.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Political,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(17),
            category: Category::Political,
            base_weight: 100,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustApproval { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "The state praised the agency's work.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Political,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(18),
            category: Category::Political,
            base_weight: 80,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPrestige { delta: 1 }],
            narrative: NarrativeSpec {
                headline: "A speech mentioned the agency kindly.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Political,
            },
            follow_ups: vec![],
        },
        // ------------------------------------------------------------- financial
        EventDef {
            id: EventId(19),
            category: Category::Financial,
            base_weight: 170,
            preconditions: vec![],
            weight_modifiers: vec![BROKE],
            effects: vec![Effect::AdjustTreasury {
                delta: Money::from_credits(-25_000),
            }],
            narrative: NarrativeSpec {
                headline: "An unexpected bill arrived.".to_owned(),
                body: Some("It had been outstanding for two years.".to_owned()),
                severity: Severity::Major,
                category: Category::Financial,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(20),
            category: Category::Financial,
            base_weight: 120,
            preconditions: vec![],
            weight_modifiers: vec![BROKE],
            effects: vec![Effect::AdjustTreasury {
                delta: Money::from_credits(-8_000),
            }],
            narrative: NarrativeSpec {
                headline: "A contractor asked for payment early.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Financial,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(21),
            category: Category::Financial,
            base_weight: 90,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustTreasury {
                delta: Money::from_credits(20_000),
            }],
            narrative: NarrativeSpec {
                headline: "A grant was awarded.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Financial,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(22),
            category: Category::Financial,
            base_weight: 70,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPrestige { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "An audit was requested.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Financial,
            },
            follow_ups: vec![],
        },
        // ------------------------------------------------------------ environmental
        EventDef {
            id: EventId(23),
            category: Category::Environmental,
            base_weight: 150,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllFatigue { delta: 3 }],
            narrative: NarrativeSpec {
                headline: "The weather turned.".to_owned(),
                body: Some("Everyone worked late, and the ground crews worked longer.".to_owned()),
                severity: Severity::Minor,
                category: Category::Environmental,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(24),
            category: Category::Environmental,
            base_weight: 110,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllHealth { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A heatwave went on too long.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Environmental,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(25),
            category: Category::Environmental,
            base_weight: 90,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllCondition { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A storm came ashore and took a roof.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Environmental,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(26),
            category: Category::Environmental,
            base_weight: 80,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllMorale { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A launch window closed.".to_owned(),
                body: Some("The work waits, and the work does not like waiting.".to_owned()),
                severity: Severity::Minor,
                category: Category::Environmental,
            },
            follow_ups: vec![],
        },
        // -------------------------------------------------------------- public
        EventDef {
            id: EventId(27),
            category: Category::Public,
            base_weight: 150,
            preconditions: vec![],
            weight_modifiers: vec![UNPOPULAR],
            effects: vec![Effect::AdjustPublicOpinion { delta: -4 }],
            narrative: NarrativeSpec {
                headline: "The press found an angle.".to_owned(),
                body: Some("It was not a serious matter, and it was in the papers.".to_owned()),
                severity: Severity::Minor,
                category: Category::Public,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(28),
            category: Category::Public,
            base_weight: 110,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPublicOpinion { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "A television programme asked awkward questions.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Public,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(29),
            category: Category::Public,
            base_weight: 100,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPrestige { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "A piece about the agency was favourable.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Public,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(30),
            category: Category::Public,
            base_weight: 80,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPublicOpinion { delta: 1 }],
            narrative: NarrativeSpec {
                headline: "A local paper mentioned the agency kindly.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Public,
            },
            follow_ups: vec![],
        },
        // ------------------------------------------------------------- external
        EventDef {
            id: EventId(31),
            category: Category::External,
            base_weight: 120,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPublicOpinion { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "Somebody else got there first.".to_owned(),
                body: Some("Their announcement made the evening news.".to_owned()),
                severity: Severity::Minor,
                category: Category::External,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(32),
            category: Category::External,
            base_weight: 90,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPrestige { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A rival agency poached a scientist.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::External,
            },
            follow_ups: vec![FollowUp {
                event: EventId(33),
                in_months: 3,
                condition: None,
            }],
        },
        EventDef {
            id: EventId(33),
            category: Category::External,
            base_weight: 80,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllLoyalty { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "The loss was still being talked about.".to_owned(),
                body: Some("A team that has been poached does not forget it.".to_owned()),
                severity: Severity::Minor,
                category: Category::External,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(34),
            category: Category::External,
            base_weight: 70,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllMorale { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "A contract went elsewhere.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::External,
            },
            follow_ups: vec![],
        },
    ]
});

/// The catalogue.
#[must_use]
pub fn catalogue() -> &'static [EventDef] {
    &CATALOGUE
}

/// The event with the given identifier.
#[must_use]
pub fn event(id: EventId) -> Option<&'static EventDef> {
    CATALOGUE.iter().find(|event| event.id == id)
}
