//! The order palette: the modal where the player gives an order.
//!
//! **The palette is how the player acts.** Until this existed the player could
//! watch a year happen and could not touch it.
//!
//! # The price
//!
//! **The price is shown before the order is confirmed, and it is always the same
//! three lines**: how long it takes, what it costs, and what it does to
//! goodwill. An order the player cannot see the price of is not a decision.
//!
//! # The refusal
//!
//! **An order that is refused is refused here, at the moment it is given**, with
//! a reason. The player is never told three weeks later that something was
//! impossible.
//!
//! # No free text
//!
//! Every parameter is a **choice from a list**, because the interface has no
//! text field. A project's title is generated from its discipline. That is a
//! limitation of the MVP, and it is recorded as one.

use domain::facility::{BuildMethod, BuildScale, ConstructionPlan, FacilityType, Financing, Site};
use domain::ids::{FacilityId, PersonId, ProgrammeId, ResearchProjectId};
use domain::money::Money;
use domain::order::{KnowledgeMode, OrderKind};
use domain::person::{Assignment, ProgrammeRole, Role};
use domain::programme::{DeadlineAudience, ProgrammeCategory};
use domain::research::{CapabilityId, DisciplineId};
use domain::state::CampaignState;
use domain::time::Instant;
use ratatui::text::Line;
use ratatui::widgets::Block;

use sim::balance;

/// A choice the player can make for one parameter of an order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Choice {
    /// A member of staff, by name.
    Person(PersonId),
    /// A role, by name.
    Role(Role),
    /// A salary, in credits.
    Salary(Money),
    /// Something to be assigned to.
    Target(Assignment),
    /// A field of research.
    Discipline(DisciplineId),
    /// A capability the agency has developed.
    Capability(CapabilityId),
    /// A course to send someone on.
    Course(&'static str),
    /// A way to build.
    Method(BuildMethod),
    /// A size to build.
    Scale(BuildScale),
    /// A kind of facility.
    Facility(FacilityType),
    /// How to release a result.
    Mode(KnowledgeMode),
    /// A programme.
    Programme(ProgrammeId),
    /// A kind of vehicle.
    Archetype(domain::vehicle::ArchetypeId),
    /// What a programme is for.
    Category(ProgrammeCategory),
    /// A horizon, in months from now.
    Horizon(u32),
    /// Who a deadline was promised to.
    Audience(DeadlineAudience),
    /// An objective, and the programme it belongs to.
    Objective(ProgrammeId, domain::ids::ObjectiveId),
}

impl Choice {
    /// The choice, as the player reads it.
    #[must_use]
    pub fn label(&self, state: &CampaignState) -> String {
        match self {
            Self::Person(id) => state
                .people
                .get(id)
                .map_or("Someone who has left".to_owned(), |person| {
                    person.name.clone()
                }),
            Self::Role(role) => role.label().to_owned(),
            Self::Salary(salary) => format!("{} a month", salary.credits()),
            Self::Target(target) => target.description().clone(),
            Self::Discipline(id) => domain::research::discipline_name(*id).to_owned(),
            Self::Capability(id) => domain::research::capability_name(*id).to_owned(),
            Self::Course(course) => (*course).to_owned(),
            Self::Method(method) => method.label().to_owned(),
            Self::Scale(scale) => scale.label().to_owned(),
            Self::Facility(kind) => kind.label().to_owned(),
            Self::Mode(mode) => mode.label().to_owned(),
            Self::Archetype(archetype) => domain::vehicle::archetype_name(*archetype).to_owned(),
            Self::Programme(id) => state
                .programmes
                .get(id)
                .map_or("A programme that has closed".to_owned(), |programme| {
                    programme.title.clone()
                }),
            Self::Category(category) => category.label().to_owned(),
            Self::Horizon(months) => format!("in {months} months"),
            Self::Audience(audience) => format!("promised to {}", audience.label()),
            Self::Objective(programme, objective) => state
                .programmes
                .get(programme)
                .and_then(|programme| programme.objectives.iter().find(|o| o.id == *objective))
                .map_or("An objective that has closed".to_owned(), |objective| {
                    objective.description.clone()
                }),
        }
    }
}

/// One parameter of an order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Parameter {
    /// What the parameter is called.
    pub label: &'static str,
    /// The options, in the order they are offered.
    pub options: Vec<Choice>,
    /// Which option is chosen.
    pub selected: usize,
}

impl Parameter {
    /// The chosen option.
    #[must_use]
    pub fn chosen(&self) -> Option<&Choice> {
        self.options.get(self.selected)
    }

    /// Moves the selection on.
    pub fn next(&mut self) {
        if !self.options.is_empty() {
            self.selected = (self.selected + 1) % self.options.len();
        }
    }

    /// Moves the selection back.
    pub fn previous(&mut self) {
        if !self.options.is_empty() {
            self.selected = if self.selected == 0 {
                self.options.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}

/// The order kinds the palette offers.
///
/// **Only the kinds that have handlers.** An order with no handler is refused
/// at issue, and offering it would be offering the player a dead end.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recipe {
    /// Hire someone.
    Hire,
    /// Let someone go.
    Fire,
    /// Put someone on something.
    Assign,
    /// Take someone off something.
    Unassign,
    /// Give someone a step up.
    Promote,
    /// Put someone through a course.
    Train,
    /// Start a research project.
    StartProject,
    /// Start a test campaign.
    StartTestCampaign,
    /// Begin building a facility.
    BuildFacility,
    /// Design a vehicle.
    DesignVehicle,
    /// Open a programme.
    CreateProgramme,
    /// Add a dated objective to a programme.
    SetObjective,
    /// Set a programme's budget.
    SetBudget,
    /// Push a deadline back.
    PostponeDeadline,
}

impl Recipe {
    /// Every recipe, in the order the palette lists them.
    pub const ALL: [Self; 14] = [
        Self::Hire,
        Self::Fire,
        Self::Assign,
        Self::Unassign,
        Self::Promote,
        Self::Train,
        Self::StartProject,
        Self::StartTestCampaign,
        Self::BuildFacility,
        Self::DesignVehicle,
        Self::CreateProgramme,
        Self::SetObjective,
        Self::SetBudget,
        Self::PostponeDeadline,
    ];

    /// The name of the order, as the player reads it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hire => "Hire",
            Self::Fire => "Fire",
            Self::Assign => "Assign",
            Self::Unassign => "Unassign",
            Self::Promote => "Promote",
            Self::Train => "Train",
            Self::StartProject => "Start a research project",
            Self::StartTestCampaign => "Start a test campaign",
            Self::BuildFacility => "Build a facility",
            Self::DesignVehicle => "Design a vehicle",
            Self::CreateProgramme => "Open a programme",
            Self::SetObjective => "Set a deadline",
            Self::SetBudget => "Set a budget",
            Self::PostponeDeadline => "Push a deadline back",
        }
    }

    /// The topic in the manual that explains this order.
    ///
    /// **The palette says what the order is for, and the manual says why.**
    /// Both come from the same store, so the two can never disagree.
    #[must_use]
    pub const fn topic(self) -> &'static str {
        match self {
            Self::Hire => "order.hire",
            Self::Fire => "order.fire",
            Self::Assign => "order.assign",
            Self::Unassign => "order.unassign",
            Self::Promote => "order.promote",
            Self::Train => "order.train",
            Self::StartProject => "order.start_project",
            Self::StartTestCampaign => "order.start_test_campaign",
            Self::BuildFacility => "order.build_facility",
            Self::DesignVehicle => "order.design_vehicle",
            Self::CreateProgramme => "order.create_programme",
            Self::SetObjective => "order.set_objective",
            Self::SetBudget => "order.set_budget",
            Self::PostponeDeadline => "order.postpone_deadline",
        }
    }

    /// The one-line explanation of this order.
    #[must_use]
    pub fn summary(self) -> Option<&'static str> {
        crate::manual::summary(self.topic())
    }

    /// Whether the recipe can be given at all, given the campaign.
    ///
    /// **A test campaign needs a capability to test** — one the agency has
    /// developed — so the recipe is not offered until it has one.
    #[must_use]
    pub fn available(self, state: &CampaignState) -> bool {
        match self {
            Self::StartTestCampaign => !state.unlocked.is_empty(),
            // **An order about a programme needs a programme.**
            Self::SetObjective | Self::SetBudget | Self::PostponeDeadline => {
                !state.programmes.is_empty()
            }
            _ => true,
        }
    }

    /// The parameters the order needs, given the campaign.
    #[must_use]
    pub fn parameters(self, state: &CampaignState) -> Vec<Parameter> {
        match self {
            Self::Hire => hire_parameters(),
            Self::Fire | Self::Unassign | Self::Promote => vec![Parameter {
                label: "Person",
                options: people(state),
                selected: 0,
            }],
            Self::Assign => vec![
                Parameter {
                    label: "Person",
                    options: people(state),
                    selected: 0,
                },
                Parameter {
                    label: "To",
                    options: targets(state),
                    selected: 0,
                },
            ],
            Self::Train => vec![
                Parameter {
                    label: "Person",
                    options: people(state),
                    selected: 0,
                },
                Parameter {
                    label: "Course",
                    options: balance::COURSES
                        .iter()
                        .copied()
                        .map(Choice::Course)
                        .collect(),
                    selected: 0,
                },
            ],
            Self::StartProject => vec![
                Parameter {
                    label: "Field",
                    options: domain::research::DISCIPLINES
                        .iter()
                        .map(|discipline| Choice::Discipline(discipline.id))
                        .collect(),
                    selected: 0,
                },
                Parameter {
                    label: "Lead",
                    options: people(state),
                    selected: 0,
                },
            ],
            Self::StartTestCampaign => vec![Parameter {
                label: "Capability",
                options: state
                    .unlocked
                    .iter()
                    .copied()
                    .map(Choice::Capability)
                    .collect(),
                selected: 0,
            }],
            // **The programme orders take their parameters from lists**, like
            // every other order, because the palette has no free text.
            Self::DesignVehicle => vec![Parameter {
                label: "Kind",
                options: domain::vehicle::ARCHETYPES
                    .iter()
                    .copied()
                    .map(Choice::Archetype)
                    .collect(),
                selected: 0,
            }],
            Self::CreateProgramme => programme_parameters(Self::CreateProgramme, state),
            Self::SetObjective => programme_parameters(Self::SetObjective, state),
            Self::SetBudget => programme_parameters(Self::SetBudget, state),
            Self::PostponeDeadline => programme_parameters(Self::PostponeDeadline, state),
            Self::BuildFacility => vec![
                Parameter {
                    label: "Kind",
                    options: FACILITY_TYPES
                        .iter()
                        .copied()
                        .map(Choice::Facility)
                        .collect(),
                    selected: 0,
                },
                Parameter {
                    label: "Method",
                    options: BUILD_METHODS.iter().copied().map(Choice::Method).collect(),
                    selected: 0,
                },
                Parameter {
                    label: "Scale",
                    options: BUILD_SCALES.iter().copied().map(Choice::Scale).collect(),
                    selected: 0,
                },
            ],
        }
    }

    /// Builds the order from the chosen parameters.
    #[must_use]
    pub fn build(self, parameters: &[Parameter]) -> Option<OrderKind> {
        let first = parameters.first().and_then(Parameter::chosen);
        match self {
            Self::Hire => {
                let role = match first {
                    Some(Choice::Role(role)) => *role,
                    _ => return None,
                };
                let salary = parameters
                    .get(1)
                    .and_then(Parameter::chosen)
                    .and_then(|choice| match choice {
                        Choice::Salary(salary) => Some(*salary),
                        _ => None,
                    })
                    .unwrap_or_else(|| balance::starting_salary(role));
                Some(OrderKind::Hire { role, salary })
            }
            // **A vehicle is designed with a default set of sliders**, and the
            // player changes the sliders on the vehicles screen.
            Self::DesignVehicle => {
                let archetype = match first {
                    Some(Choice::Archetype(archetype)) => *archetype,
                    _ => return None,
                };
                Some(OrderKind::DesignVehicle {
                    archetype,
                    name: format!("{} {}", domain::vehicle::archetype_name(archetype), 1),
                    sliders: default_sliders(archetype),
                })
            }
            // **The programme orders are built by their own function**, because
            // they take their parameters from lists like every other order.
            Self::CreateProgramme
            | Self::SetObjective
            | Self::SetBudget
            | Self::PostponeDeadline => build_programme(self, parameters, first),
            Self::Fire => person_order(first, |person| OrderKind::Fire { person }),
            Self::Unassign => person_order(first, |person| OrderKind::Unassign { person }),
            Self::Promote => person_order(first, |person| OrderKind::Promote { person }),
            Self::Assign => {
                let person = person_of(first)?;
                let to = parameters
                    .get(1)
                    .and_then(Parameter::chosen)
                    .and_then(|choice| match choice {
                        Choice::Target(target) => Some(target.clone()),
                        _ => None,
                    })?;
                Some(OrderKind::Assign { person, to })
            }
            Self::Train => {
                let person = person_of(first)?;
                let course = parameters
                    .get(1)
                    .and_then(Parameter::chosen)
                    .and_then(|choice| match choice {
                        Choice::Course(course) => Some(*course),
                        _ => None,
                    })?;
                Some(OrderKind::Train {
                    person,
                    course: course.to_owned(),
                })
            }
            Self::StartProject => {
                let discipline = match first {
                    Some(Choice::Discipline(discipline)) => *discipline,
                    _ => return None,
                };
                let lead = parameters
                    .get(1)
                    .and_then(Parameter::chosen)
                    .and_then(|choice| match choice {
                        Choice::Person(person) => Some(*person),
                        _ => None,
                    })?;
                // **The title is generated.** The interface has no text field,
                // and a project's title is display-only.
                let title = format!("{} research", domain::research::discipline_name(discipline));
                Some(OrderKind::StartProject {
                    discipline,
                    principal_investigator: lead,
                    title,
                })
            }
            Self::StartTestCampaign => first.and_then(|choice| match choice {
                Choice::Capability(capability) => Some(OrderKind::StartTestCampaign {
                    capability: *capability,
                }),
                _ => None,
            }),
            Self::BuildFacility => build_facility(parameters, first),
        }
    }
}

/// Builds the order to build a facility.
fn build_facility(parameters: &[Parameter], first: Option<&Choice>) -> Option<OrderKind> {
    let kind = match first {
        Some(Choice::Facility(kind)) => *kind,
        _ => return None,
    };
    let method = chosen_of(parameters, 1, |choice| match choice {
        Choice::Method(method) => Some(*method),
        _ => None,
    })?;
    let scale = chosen_of(parameters, 2, |choice| match choice {
        Choice::Scale(scale) => Some(*scale),
        _ => None,
    })?;
    let plan = ConstructionPlan {
        method,
        scale,
        site: Site {
            name: "The Core".to_owned(),
            publicly_visible: false,
        },
        financing: Financing::StateBudget,
        total_cost: balance::facility_base_cost(kind)
            .multiply_fraction(i64::from(scale.cost_multiplier_bp()), 10_000)
            .multiply_fraction(i64::from(method.cost_multiplier_bp()), 10_000),
        build_months: balance::facility_base_months(kind),
    };
    Some(OrderKind::BuildFacility { kind, plan })
}

/// The chosen option of the parameter at an index, read through a mapping.
fn chosen_of<T>(
    parameters: &[Parameter],
    index: usize,
    map: impl Fn(&Choice) -> Option<T>,
) -> Option<T> {
    parameters
        .get(index)
        .and_then(Parameter::chosen)
        .and_then(map)
}

/// The person a chosen option names.
fn person_of(choice: Option<&Choice>) -> Option<PersonId> {
    match choice {
        Some(Choice::Person(person)) => Some(*person),
        _ => None,
    }
}

/// An order about one person.
fn person_order<F: Fn(PersonId) -> OrderKind>(
    choice: Option<&Choice>,
    order: F,
) -> Option<OrderKind> {
    person_of(choice).map(order)
}

/// The facility types the palette offers.
const FACILITY_TYPES: [FacilityType; 4] = [
    FacilityType::Laboratory,
    FacilityType::TestStand,
    FacilityType::FabricationPlant,
    FacilityType::LaunchComplex,
];

/// The build methods the palette offers.
const BUILD_METHODS: [BuildMethod; 4] = [
    BuildMethod::FastCheap,
    BuildMethod::Standard,
    BuildMethod::Robust,
    BuildMethod::Experimental,
];

/// The build scales the palette offers.
const BUILD_SCALES: [BuildScale; 4] = [
    BuildScale::Small,
    BuildScale::Standard,
    BuildScale::Large,
    BuildScale::Oversized,
];

/// The people the player can choose from.
/// The horizons a deadline can be set at, in months.
///
/// **A list, not a free date.** The palette has no date picker, and a
/// deadline the player cannot set is a deadline they will not set.
const HORIZONS: [u32; 6] = [3, 6, 12, 18, 24, 36];

/// The budgets a programme can be given, in credits a month.
const BUDGETS: [i64; 5] = [5_000, 10_000, 25_000, 50_000, 100_000];

/// Builds one of the programme orders.
///
/// **The target is half the horizon and the commitment is the whole of it**, so
/// the player is given a slack rather than being asked to invent one.
fn build_programme(
    recipe: Recipe,
    parameters: &[Parameter],
    first: Option<&Choice>,
) -> Option<OrderKind> {
    match recipe {
        Recipe::CreateProgramme => {
            let category = match first {
                Some(Choice::Category(category)) => *category,
                _ => return None,
            };
            Some(OrderKind::CreateProgramme {
                title: category.label().to_owned(),
                category,
            })
        }
        Recipe::SetObjective => {
            let programme = match first {
                Some(Choice::Programme(id)) => *id,
                _ => return None,
            };
            let months = horizon(parameters, 1)?;
            let audience = match parameters.get(2).and_then(Parameter::chosen) {
                Some(Choice::Audience(audience)) => *audience,
                _ => return None,
            };
            let target = Instant::from_months(months / 2);
            let committed = Instant::from_months(months);
            Some(OrderKind::SetObjective {
                programme,
                description: format!("The work, promised in {months} months."),
                commitment: domain::programme::Commitment::new(
                    domain::ids::CommitmentId(0),
                    Some(target),
                    Some(committed),
                    audience,
                ),
                kind: domain::programme::ObjectiveKind::Primary,
            })
        }
        Recipe::SetBudget => {
            let programme = match first {
                Some(Choice::Programme(id)) => *id,
                _ => return None,
            };
            let allocation = match parameters.get(1).and_then(Parameter::chosen) {
                Some(Choice::Salary(salary)) => *salary,
                _ => return None,
            };
            Some(OrderKind::SetBudget {
                programme,
                allocation,
            })
        }
        Recipe::PostponeDeadline => {
            let (programme, objective) = match first {
                Some(Choice::Objective(programme, objective)) => (*programme, *objective),
                _ => return None,
            };
            let months = horizon(parameters, 1)?;
            Some(OrderKind::PostponeDeadline {
                programme,
                objective,
                to: Instant::from_months(months),
            })
        }
        _ => None,
    }
}

/// The horizon chosen at the given parameter, in months.
fn horizon(parameters: &[Parameter], index: usize) -> Option<u64> {
    match parameters.get(index).and_then(Parameter::chosen) {
        Some(Choice::Horizon(months)) => Some(u64::from(*months)),
        _ => None,
    }
}
/// The parameters of a hire.
///
/// **The salary is the role's starting salary**, and the player can pick a
/// higher one: a bigger salary attracts a better candidate, at a bigger bill.
fn hire_parameters() -> Vec<Parameter> {
    vec![
        Parameter {
            label: "Role",
            options: Role::ALL.iter().copied().map(Choice::Role).collect(),
            selected: 0,
        },
        Parameter {
            label: "Salary",
            options: Role::ALL
                .iter()
                .copied()
                .map(|role| Choice::Salary(balance::starting_salary(role)))
                .collect(),
            selected: 0,
        },
    ]
}
/// The sliders a design starts with.
///
/// **A sensible default for the archetype**, so the player is adjusting a
/// design rather than building one out of nothing.
fn default_sliders(archetype: domain::vehicle::ArchetypeId) -> domain::vehicle::SliderValues {
    let mut sliders = domain::vehicle::SliderValues::zero();
    for slider in domain::vehicle::Slider::ALL {
        let (floor, _) = domain::vehicle::archetype_band(archetype, slider);
        slider.set(&mut sliders, floor);
    }
    sliders
}

/// The parameters of one of the programme orders.
///
/// **Every parameter is a choice from a list**, because the palette has no free
/// text and no date picker. A deadline is a horizon, and the slack is half of
/// it.
fn programme_parameters(recipe: Recipe, state: &CampaignState) -> Vec<Parameter> {
    match recipe {
        Recipe::CreateProgramme => vec![Parameter {
            label: "Kind",
            options: domain::programme::ProgrammeCategory::ALL
                .iter()
                .copied()
                .map(Choice::Category)
                .collect(),
            selected: 0,
        }],
        Recipe::SetObjective => vec![
            Parameter {
                label: "Programme",
                options: programmes(state),
                selected: 0,
            },
            Parameter {
                label: "In",
                options: HORIZONS.iter().copied().map(Choice::Horizon).collect(),
                selected: 0,
            },
            Parameter {
                label: "Promised to",
                options: DeadlineAudience::ALL
                    .iter()
                    .copied()
                    .map(Choice::Audience)
                    .collect(),
                selected: 0,
            },
        ],
        Recipe::SetBudget => vec![
            Parameter {
                label: "Programme",
                options: programmes(state),
                selected: 0,
            },
            Parameter {
                label: "Budget",
                options: BUDGETS
                    .iter()
                    .copied()
                    .map(|credits| Choice::Salary(Money::from_credits(credits)))
                    .collect(),
                selected: 0,
            },
        ],
        Recipe::PostponeDeadline => vec![
            Parameter {
                label: "Objective",
                options: objectives(state),
                selected: 0,
            },
            Parameter {
                label: "In",
                options: HORIZONS.iter().copied().map(Choice::Horizon).collect(),
                selected: 0,
            },
        ],
        _ => Vec::new(),
    }
}
/// The programmes the agency is running.
fn programmes(state: &CampaignState) -> Vec<Choice> {
    state
        .programmes
        .values()
        .map(|programme| Choice::Programme(programme.id))
        .collect()
}

/// Every objective in the agency, with the programme it belongs to.
fn objectives(state: &CampaignState) -> Vec<Choice> {
    state
        .programmes
        .values()
        .flat_map(|programme| {
            programme
                .objectives
                .iter()
                .map(move |objective| Choice::Objective(programme.id, objective.id))
        })
        .collect()
}

fn people(state: &CampaignState) -> Vec<Choice> {
    state
        .people
        .values()
        .map(|person| Choice::Person(person.id))
        .collect()
}

/// The things a person can be put on.
fn targets(state: &CampaignState) -> Vec<Choice> {
    let mut targets = vec![Choice::Target(Assignment::Unassigned)];
    for programme in state.programmes.values() {
        targets.push(Choice::Target(Assignment::Programme {
            programme: programme.id,
            role: ProgrammeRole::Member,
        }));
    }
    for project in state.projects.values() {
        targets.push(Choice::Target(Assignment::Project {
            project: project.id,
        }));
    }
    for campaign in state.test_campaigns.values() {
        targets.push(Choice::Target(Assignment::TestCampaign {
            campaign: campaign.id,
        }));
    }
    for facility in state.facilities.values() {
        targets.push(Choice::Target(Assignment::Facility {
            facility: facility.id,
        }));
    }
    targets
}

/// The price of an order: how long it takes, what it costs, and the goodwill.
///
/// **The three lines are always the same**, so the player learns to read them.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Price {
    /// How long the order takes to land.
    pub lead_time: domain::time::Duration,
    /// What it costs.
    pub cost: Money,
    /// What it does to the goodwill.
    pub goodwill: i16,
}

/// The price of an order, given the campaign and the instant it is given.
#[must_use]
pub fn price(kind: &OrderKind, state: &CampaignState, issued_at: Instant) -> Option<Price> {
    let lead_time = sim::orders::lead_time(kind, state, issued_at);
    let cost =
        sim::orders::adjusted_cost(sim::orders::cost(kind, state), &state.political.goodwill);
    let goodwill = sim::orders::goodwill(kind);
    Some(Price {
        lead_time,
        cost,
        goodwill,
    })
}

/// The ids the palette's targets refer to, for the interface.
pub type TargetId = ProgrammeId;

/// A research project id, for the interface.
pub type ProjectId = ResearchProjectId;

/// A facility id, for the interface.
pub type FacilityKind = FacilityId;

/// The open palette.
///
/// **Two views.** The first lists the orders on offer; the second is the form,
/// with the parameters and the price. The player sees the price before they
/// confirm, every time.
#[derive(Clone, Debug)]
pub struct Palette {
    /// The orders on offer.
    recipes: Vec<Recipe>,
    /// Which order is highlighted.
    recipe: usize,
    /// The parameters of the chosen order.
    parameters: Vec<Parameter>,
    /// Which parameter is highlighted.
    parameter: usize,
    /// Whether the parameters are being shown.
    filling: bool,
}

impl Palette {
    /// Opens the palette, given the campaign.
    #[must_use]
    pub fn new(state: &CampaignState) -> Self {
        let recipes: Vec<Recipe> = Recipe::ALL
            .into_iter()
            .filter(|recipe| recipe.available(state))
            .collect();
        Self {
            recipes,
            recipe: 0,
            parameters: Vec::new(),
            parameter: 0,
            filling: false,
        }
    }

    /// Whether the palette is showing the form rather than the list.
    #[must_use]
    pub const fn is_filling(&self) -> bool {
        self.filling
    }

    /// The order kind the player is choosing.
    #[must_use]
    pub fn chosen_recipe(&self) -> Option<Recipe> {
        self.recipes.get(self.recipe).copied()
    }

    /// The parameters of the chosen order.
    #[must_use]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    /// Which parameter is highlighted.
    #[must_use]
    pub const fn parameter(&self) -> usize {
        self.parameter
    }

    /// The orders on offer, and which is highlighted.
    #[must_use]
    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Which order is highlighted.
    #[must_use]
    pub const fn recipe(&self) -> usize {
        self.recipe
    }

    /// Moves the selection up.
    pub fn up(&mut self) {
        if self.filling {
            if let Some(parameter) = self.parameters.get_mut(self.parameter) {
                parameter.previous();
            }
        } else if !self.recipes.is_empty() {
            self.recipe = if self.recipe == 0 {
                self.recipes.len() - 1
            } else {
                self.recipe - 1
            };
        }
    }

    /// Moves the selection down.
    pub fn down(&mut self) {
        if self.filling {
            if let Some(parameter) = self.parameters.get_mut(self.parameter) {
                parameter.next();
            }
        } else if !self.recipes.is_empty() {
            self.recipe = (self.recipe + 1) % self.recipes.len();
        }
    }

    /// Moves to the previous parameter.
    pub fn previous_parameter(&mut self) {
        if self.filling && !self.parameters.is_empty() {
            self.parameter = if self.parameter == 0 {
                self.parameters.len() - 1
            } else {
                self.parameter - 1
            };
        }
    }

    /// Moves to the next parameter.
    pub fn next_parameter(&mut self) {
        if self.filling && !self.parameters.is_empty() {
            self.parameter = (self.parameter + 1) % self.parameters.len();
        }
    }

    /// Confirms the current selection.
    ///
    /// On the list of orders, this moves to the form. On the form, it returns
    /// the order — **if** one can be built from the chosen parameters.
    pub fn accept(&mut self, state: &CampaignState) -> Option<OrderKind> {
        if self.filling {
            let recipe = self.chosen_recipe()?;
            return recipe.build(&self.parameters);
        }
        // Moving to the form, with the parameters filled in.
        let recipe = self.chosen_recipe()?;
        self.parameters = recipe.parameters(state);
        self.parameter = 0;
        self.filling = true;
        None
    }

    /// Goes back, or closes the palette.
    ///
    /// Returns whether the palette is still open.
    pub fn back(&mut self) -> bool {
        if self.filling {
            self.filling = false;
            self.parameters.clear();
            true
        } else {
            false
        }
    }
}

/// Draws the palette over the screen behind it.
///
/// **The price is always the same three lines**, so the player learns to read
/// them: how long it takes, what it costs, and what it does to goodwill.
pub fn render(
    palette: &Palette,
    state: &CampaignState,
    issued_at: Instant,
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
) {
    let lines = if palette.is_filling() {
        render_form(palette, state, issued_at)
    } else {
        render_list(palette)
    };

    let block = Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .title("Order");
    let paragraph = ratatui::widgets::Paragraph::new(lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });
    // **The palette is a modal, and it clears what is behind it.** Without
    // this the dashboard's text shows through the gaps in the palette, and a
    // player reading an order sees half of a project title beside it.
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(paragraph, area);
}

/// The list of orders on offer.
fn render_list(palette: &Palette) -> Vec<Line<'static>> {
    let mut lines = vec![Line::raw("  What do you want to do?"), Line::raw("")];
    for (index, recipe) in palette.recipes().iter().enumerate() {
        // **The focus is a style and a marker**, so it is findable in a
        // monochrome terminal as well as a colourful one.
        let (marker, style) = if index == palette.recipe() {
            (crate::theme::FOCUS, crate::theme::selected())
        } else {
            (crate::theme::NO_FOCUS, crate::theme::historical())
        };
        lines.push(Line::styled(format!(" {marker} {}", recipe.label()), style));
    }

    // **What the selected order is for.** The list of what is possible is not
    // the same as knowing what any of it means, and the player is about to
    // spend money and goodwill.
    if let Some(recipe) = palette.chosen_recipe() {
        lines.push(Line::raw(""));
        if let Some(summary) = recipe.summary() {
            lines.push(Line::styled(
                format!("  {summary}"),
                crate::theme::Meaning::Informational.style(),
            ));
            lines.push(Line::styled(
                "  F2 for the manual, I for this screen.",
                crate::theme::historical(),
            ));
        }
    }

    lines
}

/// The form: the parameters, the price, and the reason it might be a bad idea.
fn render_form(palette: &Palette, state: &CampaignState, issued_at: Instant) -> Vec<Line<'static>> {
    let recipe = palette.chosen_recipe();
    let mut lines = vec![
        Line::styled(
            format!(
                "  Order: {}",
                recipe.map_or_else(
                    || "Something".to_owned(),
                    |recipe| recipe.label().to_owned()
                )
            ),
            crate::theme::title(),
        ),
        Line::raw(""),
    ];

    // **What the order is for**, under its name.
    if let Some(recipe) = recipe {
        if let Some(summary) = recipe.summary() {
            lines.push(Line::styled(
                format!("  {summary}"),
                crate::theme::Meaning::Informational.style(),
            ));
            lines.push(Line::raw(""));
        }
    }

    for (index, parameter) in palette.parameters().iter().enumerate() {
        let marker = if index == palette.parameter() {
            "▸"
        } else {
            " "
        };
        lines.push(Line::raw(format!("  {marker} {:<12}", parameter.label)));
        if let Some(chosen) = parameter.chosen() {
            lines.push(Line::styled(
                format!("      {}", chosen.label(state)),
                crate::theme::emphasis(),
            ));
        }
        if parameter.options.len() > 1 {
            lines.push(Line::styled(
                format!(
                    "      ({} of {} — ← → to change)",
                    parameter.selected + 1,
                    parameter.options.len()
                ),
                crate::theme::historical(),
            ));
        }
    }
    lines.push(Line::raw(""));

    // **The price.** Three lines, always the same three.
    if let Some(recipe) = recipe {
        if let Some(kind) = recipe.build(palette.parameters()) {
            if let Some(price) = price(&kind, state, issued_at) {
                lines.push(Line::styled(
                    format!("  Takes effect  {}", describe(price.lead_time)),
                    crate::theme::title(),
                ));
                lines.push(Line::styled(
                    format!(
                        "  Cost         {}",
                        if price.cost.is_zero() {
                            "—".to_owned()
                        } else {
                            format!("{} credits", price.cost.credits())
                        }
                    ),
                    crate::theme::title(),
                ));
                lines.push(Line::styled(
                    format!("  Goodwill     {:+}", price.goodwill),
                    crate::theme::title(),
                ));
                lines.push(Line::raw(""));

                // **The ⓘ line says why the order might be a bad idea.** It is a
                // factual statement about the state, not advice about the game.
                if let Err(reason) = sim::orders::validate(&kind, state) {
                    lines.push(Line::styled(
                        format!("  ⓘ {reason}"),
                        crate::theme::Meaning::Attention.style(),
                    ));
                    lines.push(Line::raw(""));
                }
            }
        }
    }
    lines.push(Line::styled(
        "  [ Enter to confirm ]   [ Esc to cancel ]",
        crate::theme::historical(),
    ));
    lines
}

/// A duration, as the player reads it.
fn describe(duration: domain::time::Duration) -> String {
    if duration.millis() == 0 {
        return "immediately".to_owned();
    }
    let days = duration.millis() / domain::time::MILLIS_PER_DAY;
    if days == 0 {
        let hours = duration.millis() / domain::time::MILLIS_PER_HOUR;
        return format!("{hours} hours from now");
    }
    format!("{days} days from now")
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::AgencyId;
    use domain::state::Agency;

    fn state() -> CampaignState {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        state.people.insert(
            PersonId(0),
            domain::person::Person::new(
                PersonId(0),
                "Marsha Okonjo".to_owned(),
                Role::Scientist,
                Money::from_credits(1_200),
            ),
        );
        state
    }

    #[test]
    fn a_hire_is_a_role_and_a_salary() {
        let campaign = state();
        let parameters = Recipe::Hire.parameters(&campaign);
        assert_eq!(parameters.len(), 2);
        let kind = Recipe::Hire.build(&parameters).expect("the order builds");
        assert!(matches!(kind, OrderKind::Hire { .. }));
    }

    #[test]
    fn an_assign_is_a_person_and_a_target() {
        let campaign = state();
        let parameters = Recipe::Assign.parameters(&campaign);
        assert_eq!(parameters.len(), 2);
        let kind = Recipe::Assign.build(&parameters).expect("the order builds");
        assert!(matches!(kind, OrderKind::Assign { .. }));
    }

    #[test]
    fn the_palette_only_offers_kinds_that_have_handlers() {
        for recipe in Recipe::ALL {
            let campaign = state();
            if !recipe.available(&campaign) {
                continue;
            }
            let parameters = recipe.parameters(&campaign);
            let kind = recipe.build(&parameters).expect("the order builds");
            assert!(
                sim::orders::has_handler(&kind),
                "{recipe:?} has no handler and would be refused"
            );
        }
    }

    #[test]
    fn a_test_campaign_is_not_offered_before_the_agency_has_a_capability() {
        let mut campaign = state();
        assert!(!Recipe::StartTestCampaign.available(&campaign));
        campaign.unlocked.insert(CapabilityId(1));
        assert!(Recipe::StartTestCampaign.available(&campaign));
    }

    #[test]
    fn the_price_has_three_parts() {
        let campaign = state();
        let kind = OrderKind::Hire {
            role: Role::Scientist,
            salary: Money::from_credits(1_200),
        };
        let price = price(&kind, &campaign, Instant::ZERO).expect("the price");
        assert!(price.lead_time > domain::time::Duration::ZERO);
        assert_eq!(price.cost, Money::from_credits(1_200));
        assert_eq!(price.goodwill, balance::GOODWILL_HIRE);
    }
}
