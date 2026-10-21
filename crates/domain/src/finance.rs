//! Budgets, the treasury, and the income sources.
//!
//! `Budget` distinguishes money spent from money committed but not yet drawn
//! down, because pledged money is not free money. The `IncomeSource` enum exists
//! in full from the start so that adding a revenue source is not a change to the
//! aggregate's shape. See `docs/DOMAIN_MODEL.md` §12.

use serde::{Deserialize, Serialize};

use crate::money::Money;

/// A pot of money set aside for something.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Budget {
    /// The amount set aside.
    pub allocation: Money,
    /// The amount already drawn down.
    pub spent: Money,
    /// The amount pledged to work not yet done.
    pub committed: Money,
}

impl Budget {
    /// A budget with nothing in it.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            allocation: Money::ZERO,
            spent: Money::ZERO,
            committed: Money::ZERO,
        }
    }

    /// A budget with an allocation and nothing spent.
    #[must_use]
    pub const fn of(allocation: Money) -> Self {
        Self {
            allocation,
            spent: Money::ZERO,
            committed: Money::ZERO,
        }
    }

    /// The money still available.
    #[must_use]
    pub const fn remaining(self) -> Money {
        Money::from_minor(
            self.allocation
                .minor()
                .saturating_sub(self.spent.minor())
                .saturating_sub(self.committed.minor()),
        )
    }

    /// Whether the budget has been overspent.
    #[must_use]
    pub const fn is_over_budget(self) -> bool {
        self.spent.minor() > self.allocation.minor()
    }
}

/// Where money comes from.
///
/// The variant list is complete from the start, and the simulation wires them
/// one at a time. Adding a source is a variant and a calculation, not a change
/// to the aggregate's shape.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum IncomeSource {
    /// The floor. What the civilisation can afford.
    CivilisationBase,
    /// A multiplier on the base: approval and the state's priorities.
    PoliticalStanding,
    /// Private capital, with conditions attached.
    Investors,
    /// Value created by what the agency has discovered.
    Discoveries,
    /// Launch services, hosted payloads, subscription data.
    Commercial,
    /// A one-off grant for a prestige objective.
    StrategicGrant,
    /// Borrowed money.
    Debt,
}

impl IncomeSource {
    /// Every source.
    pub const ALL: [Self; 7] = [
        Self::CivilisationBase,
        Self::PoliticalStanding,
        Self::Investors,
        Self::Discoveries,
        Self::Commercial,
        Self::StrategicGrant,
        Self::Debt,
    ];

    /// The source's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::CivilisationBase => "State allocation",
            Self::PoliticalStanding => "Political standing",
            Self::Investors => "Private investment",
            Self::Discoveries => "Discovery value",
            Self::Commercial => "Commercial activity",
            Self::StrategicGrant => "Strategic grant",
            Self::Debt => "Borrowing",
        }
    }
}

/// How badly the agency is in financial trouble.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum CrisisStage {
    /// Nothing wrong.
    None,
    /// A shortfall is projected. The player can see it coming.
    Warning,
    /// The state has imposed a cut.
    Austerity,
    /// Emergency loans have been taken out.
    Borrowing,
    /// Unpaid staff are leaving.
    Attrition,
    /// Programmes are being terminated.
    Cancellations,
    /// The campaign is over.
    Collapse,
}

impl CrisisStage {
    /// Every stage, in order.
    pub const ALL: [Self; 7] = [
        Self::None,
        Self::Warning,
        Self::Austerity,
        Self::Borrowing,
        Self::Attrition,
        Self::Cancellations,
        Self::Collapse,
    ];

    /// The stage's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "Solvent",
            Self::Warning => "Warning",
            Self::Austerity => "Austerity",
            Self::Borrowing => "Borrowing",
            Self::Attrition => "Attrition",
            Self::Cancellations => "Cancellations",
            Self::Collapse => "Collapse",
        }
    }

    /// The next stage down the ladder.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::None => Self::Warning,
            Self::Warning => Self::Austerity,
            Self::Austerity => Self::Borrowing,
            Self::Borrowing => Self::Attrition,
            Self::Attrition => Self::Cancellations,
            Self::Cancellations | Self::Collapse => Self::Collapse,
        }
    }
}

/// A loan the agency has taken out.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Debt {
    /// How much was borrowed.
    pub principal: Money,
    /// How much is still owed.
    pub outstanding: Money,
    /// The annual interest rate, in percent.
    pub interest_percent: u32,
    /// How many months are left to run.
    pub months_remaining: u32,
}

/// The agency's money.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Treasury {
    /// What is in the bank.
    pub balance: Money,
    /// What came in last month, by source.
    pub income: Vec<(IncomeSource, Money)>,
    /// What went out last month.
    pub outgoings: Vec<(String, Money)>,
    /// The loans outstanding.
    pub debt: Vec<Debt>,
    /// How far down the crisis ladder the agency is.
    pub crisis_stage: CrisisStage,
    /// The fractional remainder carried between rate applications.
    ///
    /// The money flow is a rate, and a rate applied over a fraction of a month
    /// truncates to whole minor units. Over a long campaign that is a real
    /// amount of money.
    pub remainders: crate::scale::Remainders,
}

impl Treasury {
    /// An empty treasury.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            balance: Money::ZERO,
            income: Vec::new(),
            outgoings: Vec::new(),
            debt: Vec::new(),
            crisis_stage: CrisisStage::None,
            remainders: crate::scale::Remainders::new(),
        }
    }

    /// The total that came in last month.
    #[must_use]
    pub fn total_income(&self) -> Money {
        self.income.iter().fold(Money::ZERO, |total, (_, amount)| {
            total.checked_add(*amount).unwrap_or(total)
        })
    }

    /// The total that went out last month.
    #[must_use]
    pub fn total_outgoings(&self) -> Money {
        self.outgoings
            .iter()
            .fold(Money::ZERO, |total, (_, amount)| {
                total.checked_add(*amount).unwrap_or(total)
            })
    }

    /// The difference between income and outgoings last month.
    #[must_use]
    pub fn net(&self) -> Money {
        let income = self.total_income().minor();
        let outgoings = self.total_outgoings().minor();
        Money::from_minor(income - outgoings)
    }

    /// The difference between income and outgoings, per day.
    ///
    /// **The game runs in real time, so the player thinks in "per day".** A
    /// monthly figure is too big to read at a glance, and a daily one is the
    /// number the player is actually budgeting against.
    #[must_use]
    pub fn daily_net(&self) -> Money {
        let days = i64::try_from(crate::time::DAYS_PER_MONTH).unwrap_or(30);
        Money::from_minor(self.net().minor() / days)
    }

    /// How many months the treasury can cover the current burn rate.
    ///
    /// This is the number that turns a spreadsheet into a survival clock, and it
    /// is the single most important figure on the dashboard.
    #[must_use]
    pub fn runway_months(&self) -> u32 {
        let net = self.net();
        if !net.is_negative() {
            return u32::MAX;
        }
        let burn = -net.minor();
        if burn == 0 {
            return u32::MAX;
        }
        let balance = self.balance.minor().max(0);
        (balance / burn) as u32
    }
}

impl Default for Treasury {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_budget_knows_what_is_left() {
        let budget = Budget {
            allocation: Money::from_credits(1_000),
            spent: Money::from_credits(300),
            committed: Money::from_credits(200),
        };
        assert_eq!(budget.remaining(), Money::from_credits(500));
        assert!(!budget.is_over_budget());
    }

    #[test]
    fn an_overspent_budget_says_so() {
        let budget = Budget {
            allocation: Money::from_credits(100),
            spent: Money::from_credits(150),
            committed: Money::ZERO,
        };
        assert!(budget.is_over_budget());
    }

    #[test]
    fn the_runway_counts_months_of_burn() {
        let treasury = Treasury {
            balance: Money::from_credits(190_000),
            income: vec![(IncomeSource::CivilisationBase, Money::from_credits(120_000))],
            outgoings: vec![("Payroll".to_owned(), Money::from_credits(158_000))],
            debt: Vec::new(),
            crisis_stage: CrisisStage::None,
            remainders: crate::scale::Remainders::new(),
        };
        assert_eq!(treasury.runway_months(), 5);
    }

    #[test]
    fn a_surplus_has_infinite_runway() {
        let treasury = Treasury {
            balance: Money::from_credits(10),
            income: vec![(IncomeSource::CivilisationBase, Money::from_credits(200))],
            outgoings: vec![("Payroll".to_owned(), Money::from_credits(100))],
            debt: Vec::new(),
            crisis_stage: CrisisStage::None,
            remainders: crate::scale::Remainders::new(),
        };
        assert_eq!(treasury.runway_months(), u32::MAX);
    }

    #[test]
    fn the_crisis_ladder_only_goes_downwards() {
        let mut stage = CrisisStage::None;
        for _ in 0..10 {
            stage = stage.next();
        }
        assert_eq!(stage, CrisisStage::Collapse);
    }
}
