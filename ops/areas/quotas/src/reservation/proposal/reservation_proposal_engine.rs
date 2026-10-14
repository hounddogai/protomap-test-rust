use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation proposal calculation.
pub trait ReservationProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationProposalAdjuster {
    pub step: f64,
}

impl ReservationProposalStep for RoundingReservationProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationProposalAdjuster {
    pub cap: f64,
}

impl ReservationProposalStep for CapReservationProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProposalStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl ReservationProposalStep for ReservationProposalStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationProposal, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationProposalStep for TieredReservationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionReservationProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationProposalStep for RegionReservationProposalAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ReservationProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the reservation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationProposalEngine {
    pub steps: Vec<Rc<dyn ReservationProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationProposalEngine {
    pub fn price(&mut self, item: &ReservationProposal) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationProposalStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ReservationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_reservation_proposal_engine() -> ReservationProposalEngine {
    ReservationProposalEngine {
        steps: vec![
            Rc::new(RoundingReservationProposalAdjuster { step: 1.0 }) as Rc<dyn ReservationProposalStep>,
            Rc::new(CapReservationProposalAdjuster { cap: 49576.0 }),
            Rc::new(ReservationProposalStartsAtOffsetAdjuster { weight: 0.88 }),
            Rc::new(TieredReservationProposalAdjuster {
                bounds: vec![1399.0, 3417.0, 4248.0, 5951.0],
                rates: vec![1.0, 1.16, 0.84, 0.85],
            }),
            Rc::new(RegionReservationProposalAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.16),
                    ("north".to_string(), 1.39),
                    ("retail".to_string(), 0.92),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 93867.8,
    }
}
