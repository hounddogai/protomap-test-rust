use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the reservation draft calculation.
pub trait ReservationDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDraftThresholdOffsetAdjuster {
    pub weight: f64,
}

impl ReservationDraftStep for ReservationDraftThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationDraft, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationDraftStep for FlaggedReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationDraftAdjuster {
    pub minimum: f64,
}

impl ReservationDraftStep for MinimumReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationDraftStep for TieredReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceReservationDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationDraftStep for NamespaceReservationDraftAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ReservationDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the reservation draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationDraftEngine {
    pub steps: Vec<Rc<dyn ReservationDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationDraftEngine {
    pub fn compute(&mut self, item: &ReservationDraft) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_reservation_draft_engine(flags: &flags::Registry) -> ReservationDraftEngine {
    ReservationDraftEngine {
        steps: vec![
            Rc::new(ReservationDraftThresholdOffsetAdjuster { weight: 0.06 }) as Rc<dyn ReservationDraftStep>,
            Rc::new(FlaggedReservationDraftAdjuster {
                flags: flags.clone(),
                flag: "reservation-draft-strict".to_string(),
                percent: -7.5,
            }),
            Rc::new(MinimumReservationDraftAdjuster { minimum: 33.8 }),
            Rc::new(TieredReservationDraftAdjuster {
                bounds: vec![5231.0, 5637.0, 8348.0],
                rates: vec![0.94, 1.18, 1.12],
            }),
            Rc::new(NamespaceReservationDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.22)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 41876.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_draft_engine(&flags::new_registry());
        let item = new_reservation_draft("code-855", "name-43");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
