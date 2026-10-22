//! The invariant catalogue: the rules that keep the state coherent.
//!
//! A domain type that can be constructed in an invalid state is a bug, so these
//! are enforced by checked constructors and assertions rather than documented
//! and hoped for. See `docs/DOMAIN_MODEL.md` §18.

use std::fmt;

use crate::chronicle::SubjectRef;
use crate::state::CampaignState;

/// A way in which the campaign state is not coherent.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InvariantViolation {
    /// A person's attributes are outside their permitted range.
    AttributeOutOfRange {
        /// The person.
        person: String,
        /// Which attribute.
        attribute: &'static str,
        /// Its value.
        value: i32,
    },
    /// A person's morale, loyalty, or fame is outside its permitted range.
    StateOutOfRange {
        /// The person.
        person: String,
        /// Which value.
        field: &'static str,
        /// Its value.
        value: i32,
    },
    /// A commitment's target is later than its committed date.
    CommitmentInverted {
        /// What the commitment is for.
        description: String,
    },
    /// A budget has been overspent without saying so.
    BudgetOverrun {
        /// What the budget is for.
        description: String,
    },
    /// A facility under construction also has a specification.
    FacilityHalfBuilt {
        /// The facility.
        facility: String,
    },
    /// A chronicle entry refers to something that does not exist.
    DanglingSubject {
        /// The entry's headline.
        headline: String,
    },
    /// Two people have the same name.
    DuplicateName {
        /// The name.
        name: String,
    },
}

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AttributeOutOfRange {
                person,
                attribute,
                value,
            } => write!(f, "{person} has an out-of-range {attribute}: {value}"),
            Self::StateOutOfRange {
                person,
                field,
                value,
            } => write!(f, "{person} has an out-of-range {field}: {value}"),
            Self::CommitmentInverted { description } => {
                write!(
                    f,
                    "{description} has a target date later than its commitment"
                )
            }
            Self::BudgetOverrun { description } => {
                write!(f, "{description} has overspent its budget")
            }
            Self::FacilityHalfBuilt { facility } => {
                write!(
                    f,
                    "{facility} is under construction but has a specification"
                )
            }
            Self::DanglingSubject { headline } => {
                write!(
                    f,
                    "the chronicle entry '{headline}' refers to something that no longer exists"
                )
            }
            Self::DuplicateName { name } => write!(f, "two people are both called {name}"),
        }
    }
}

/// Checks every invariant in the catalogue.
///
/// Returns the first violation found. The order is fixed, so the same broken
/// state always reports the same problem.
pub fn check(state: &CampaignState) -> Result<(), InvariantViolation> {
    check_people(state)?;
    check_commitments(state)?;
    check_budgets(state)?;
    check_facilities(state)?;
    check_chronicle(state)?;
    Ok(())
}

/// Every person's attributes and state are in range, and no two share a name.
fn check_people(state: &CampaignState) -> Result<(), InvariantViolation> {
    let mut names: Vec<&str> = Vec::new();
    for person in state.people.values() {
        for (attribute, value) in [
            ("technical", person.attributes.technical),
            ("creative", person.attributes.creative),
            ("leadership", person.attributes.leadership),
            ("physical", person.attributes.physical),
            ("diplomatic", person.attributes.diplomatic),
        ] {
            if value > 100 {
                return Err(InvariantViolation::AttributeOutOfRange {
                    person: person.name.clone(),
                    attribute,
                    value: i32::from(value),
                });
            }
        }
        for (field, value) in [
            ("morale", person.state.morale),
            ("loyalty", person.state.loyalty),
            ("fame", person.state.fame),
        ] {
            if !(-25_600..=25_600).contains(&value) {
                return Err(InvariantViolation::StateOutOfRange {
                    person: person.name.clone(),
                    field,
                    value: i32::from(value),
                });
            }
        }
        // **Fatigue and health are bounded below and above** - they are not
        // signed, and the catalogue says so. The symptom of skipping them is a
        // person whose fatigue has run past the top of the scale, and every
        // comparison against it is then wrong.
        for (field, value) in [
            ("fatigue", person.state.fatigue),
            ("health", person.state.health),
        ] {
            if value > 25_600 {
                return Err(InvariantViolation::StateOutOfRange {
                    person: person.name.clone(),
                    field,
                    value: i32::from(value),
                });
            }
        }
        if names.contains(&person.name.as_str()) {
            return Err(InvariantViolation::DuplicateName {
                name: person.name.clone(),
            });
        }
        names.push(&person.name);
    }
    Ok(())
}

/// No commitment has a target later than its promised date.
fn check_commitments(state: &CampaignState) -> Result<(), InvariantViolation> {
    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            if let (Some(target), Some(committed)) =
                (objective.commitment.target, objective.commitment.committed)
            {
                if target > committed {
                    return Err(InvariantViolation::CommitmentInverted {
                        description: objective.description.clone(),
                    });
                }
            }
        }
    }
    for promise in &state.political.promises {
        if let (Some(target), Some(committed)) =
            (promise.commitment.target, promise.commitment.committed)
        {
            if target > committed {
                return Err(InvariantViolation::CommitmentInverted {
                    description: promise.description.clone(),
                });
            }
        }
    }
    Ok(())
}

/// No budget has been silently overspent.
fn check_budgets(state: &CampaignState) -> Result<(), InvariantViolation> {
    for programme in state.programmes.values() {
        if programme.budget.is_over_budget() {
            return Err(InvariantViolation::BudgetOverrun {
                description: programme.title.clone(),
            });
        }
    }
    Ok(())
}

/// No facility is half-built.
fn check_facilities(state: &CampaignState) -> Result<(), InvariantViolation> {
    for facility in state.facilities.values() {
        let building = facility.construction.is_some();
        let specified = facility.specification.capacity > 0;
        if building && specified {
            return Err(InvariantViolation::FacilityHalfBuilt {
                facility: facility.name.clone(),
            });
        }
    }
    Ok(())
}

/// Every chronicle entry refers to something that existed.
///
/// **The chronicle is a record of the past, and the past includes people who
/// have since left.** A record of somebody's resignation is about that person,
/// and it is not a dangling reference. The rule is therefore that the subject
/// either still exists or was allocated at some point — and what it catches is
/// an entry about an id that was never handed out, which is a bug rather than
/// a history.
fn check_chronicle(state: &CampaignState) -> Result<(), InvariantViolation> {
    for entry in &state.chronicle {
        for subject in &entry.subjects {
            let exists = match subject {
                SubjectRef::Programme(id) => state.programmes.contains_key(id),
                SubjectRef::Person(id) => {
                    state.people.contains_key(id) || id.0 < state.id_counters.people_allocated()
                }
                SubjectRef::Facility(id) => state.facilities.contains_key(id),
                SubjectRef::Vehicle(_) => true,
            };
            if !exists {
                return Err(InvariantViolation::DanglingSubject {
                    headline: entry.headline.clone(),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chronicle::{Category, ChronicleEntry, Severity};
    use crate::ids::{ChronicleId, PersonId};
    use crate::money::Money;
    use crate::person::{Person, Role};
    use crate::state::Agency;
    use crate::time::Instant;

    fn state() -> CampaignState {
        CampaignState::new(Agency::new(crate::ids::AgencyId(0), "SEA".to_owned(), 2039))
    }

    #[test]
    fn a_new_campaign_satisfies_every_invariant() {
        assert!(check(&state()).is_ok());
    }

    #[test]
    fn an_out_of_range_attribute_is_caught() {
        let mut campaign = state();
        let mut person = Person::new(
            PersonId(0),
            "Marsha Okonjo".to_owned(),
            Role::Scientist,
            Money::from_credits(1_200),
        );
        person.attributes.technical = 200;
        campaign.people.insert(PersonId(0), person);
        assert!(matches!(
            check(&campaign),
            Err(InvariantViolation::AttributeOutOfRange { .. })
        ));
    }

    #[test]
    fn two_people_with_the_same_name_are_caught() {
        let mut campaign = state();
        for id in 0..2 {
            campaign.people.insert(
                PersonId(id),
                Person::new(
                    PersonId(id),
                    "Marsha Okonjo".to_owned(),
                    Role::Scientist,
                    Money::from_credits(1_200),
                ),
            );
        }
        assert!(matches!(
            check(&campaign),
            Err(InvariantViolation::DuplicateName { .. })
        ));
    }

    #[test]
    fn a_dangling_chronicle_subject_is_caught() {
        let mut campaign = state();
        campaign.chronicle.push(
            ChronicleEntry::new(
                ChronicleId(0),
                Instant::ZERO,
                Category::Technical,
                "Something happened to nobody".to_owned(),
                Severity::Info,
            )
            .about(SubjectRef::Person(PersonId(99))),
        );
        assert!(matches!(
            check(&campaign),
            Err(InvariantViolation::DanglingSubject { .. })
        ));
    }

    #[test]
    fn a_chronicle_entry_about_somebody_who_has_left_is_not_dangling() {
        // **The chronicle is a record of the past, and the past includes
        // people who have since left.** A record of somebody's resignation is
        // about that person.
        let mut campaign = state();
        let id = campaign.id_counters.next_person();
        campaign.chronicle.push(
            ChronicleEntry::new(
                ChronicleId(0),
                Instant::ZERO,
                Category::Personnel,
                "Somebody resigned".to_owned(),
                Severity::Info,
            )
            .about(SubjectRef::Person(id)),
        );
        assert!(check(&campaign).is_ok());
    }

    #[test]
    fn a_violation_describes_itself_in_a_sentence() {
        let violation = InvariantViolation::DuplicateName {
            name: "Marsha Okonjo".to_owned(),
        };
        assert!(violation.to_string().contains("Marsha Okonjo"));
    }
}
