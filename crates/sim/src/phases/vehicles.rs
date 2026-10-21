//! The vehicles: designing them and changing their subsystems.
//!
//! **A vehicle is the player's decision made of parts.** The design is a set
//! of sliders, the sliders are bounded by the archetype and by what the agency
//! has developed, and the numbers that come out are derived from the sliders
//! and never stored as an authority.

use domain::chronicle::{Category, Severity};
use domain::ids::VehicleId;
use domain::order::OrderKind;
use domain::vehicle::{ArchetypeId, Slider, SliderValues, Vehicle};

use crate::tick::MonthContext;

/// The vehicles phase.
pub struct VehiclesPhase;

impl crate::tick::Phase for VehiclesPhase {
    const NAME: &'static str = "vehicles";

    fn run(context: &mut MonthContext<'_>) -> Result<(), crate::error::SimError> {
        apply_orders(context);
        Ok(())
    }
}

/// Applies the vehicle orders that have landed.
fn apply_orders(context: &mut MonthContext<'_>) {
    let orders = std::mem::take(&mut context.validated);
    for order in orders {
        if !apply_one(context, &order) {
            context.validated.push(order);
        }
    }
}

/// Applies one order, and reports whether it was one of ours.
pub fn apply_one(context: &mut MonthContext<'_>, order: &OrderKind) -> bool {
    match order {
        OrderKind::DesignVehicle {
            archetype,
            name,
            sliders,
        } => {
            design(context, *archetype, name, *sliders);
            true
        }
        OrderKind::SetSubsystem {
            vehicle,
            slider,
            value,
        } => {
            set_subsystem(context, *vehicle, *slider, *value);
            true
        }
        _ => false,
    }
}

/// Designs a vehicle.
///
/// **The sliders are bounded before the design is stored**, so the vehicle the
/// agency owns is a vehicle the agency could legally have built.
fn design(
    context: &mut MonthContext<'_>,
    archetype: ArchetypeId,
    name: &str,
    sliders: SliderValues,
) {
    let unlocked: Vec<_> = context.state.unlocked.iter().copied().collect();
    let bounded = sliders.bounded(archetype, &unlocked);
    let id = context.state.id_counters.next_vehicle();
    let mut vehicle = Vehicle::new(id, name.to_owned(), archetype, bounded);
    // **The design starts the build.** A vehicle is not a thing the agency
    // owns; it is a thing the agency is building, and it takes real time.
    vehicle.build_state = domain::vehicle::BuildState::Building;
    let cost = vehicle.derived.cost_estimate;
    context.state.vehicles.insert(id, vehicle);

    context.programme_lines.push(
        domain::report::ReportLine::new(format!("{name} was designed, and costs {cost}."))
            .with_severity(Severity::Info),
    );
    context.events.push(domain::chronicle::ChronicleEntry::new(
        context.state.next_chronicle_id(),
        context.month,
        Category::Technical,
        format!("{name} was designed"),
        Severity::Info,
    ));
}

/// Changes one of a vehicle's subsystems.
///
/// **The value is bounded the same way the design was**, because a slider the
/// agency has not developed does not become available by being edited.
fn set_subsystem(context: &mut MonthContext<'_>, vehicle: VehicleId, slider: Slider, value: u8) {
    let Some(existing) = context.state.vehicles.get(&vehicle) else {
        return;
    };
    let archetype = existing.archetype;
    let unlocked: Vec<_> = context.state.unlocked.iter().copied().collect();

    let mut values = existing.sliders;
    slider.set(&mut values, value);
    let bounded = values.bounded(archetype, &unlocked);

    let vehicle = context
        .state
        .vehicles
        .get_mut(&vehicle)
        .expect("the vehicle exists");
    vehicle.sliders = bounded;
    vehicle.derived = domain::vehicle::derive_stats(bounded, archetype);
    vehicle.cost = vehicle.derived.cost_estimate;

    let name = vehicle.name.clone();
    let cost = vehicle.cost;
    context.programme_lines.push(
        domain::report::ReportLine::new(format!(
            "{name}'s {} was changed, and it now costs {cost}.",
            slider.name()
        ))
        .with_severity(Severity::Info),
    );
}
