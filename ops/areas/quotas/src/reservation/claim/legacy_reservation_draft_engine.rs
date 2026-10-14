use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy reservation draft calculation.
pub trait LegacyReservationDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyReservationDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraftQuantityOffsetAdjuster {
    pub weight: f64,
}

impl LegacyReservationDraftStep for LegacyReservationDraftQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyReservationDraft, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyReservationDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyReservationDraftStep for TieredLegacyReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyReservationDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyReservationDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyReservationDraftStep for FlaggedLegacyReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyReservationDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy reservation draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyReservationDraftEngine {
    pub steps: Vec<Rc<dyn LegacyReservationDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyReservationDraftEngine {
    pub fn run(&mut self, item: &LegacyReservationDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyReservationDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyReservationDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_reservation_draft_engine(flags: &flags::Registry) -> LegacyReservationDraftEngine {
    LegacyReservationDraftEngine {
        steps: vec![
            Rc::new(LegacyReservationDraftQuantityOffsetAdjuster { weight: 0.37 })
                as Rc<dyn LegacyReservationDraftStep>,
            Rc::new(TieredLegacyReservationDraftAdjuster {
                bounds: vec![959.0, 975.0, 1300.0],
                rates: vec![0.93, 1.01, 0.9],
            }),
            Rc::new(FlaggedLegacyReservationDraftAdjuster {
                flags: flags.clone(),
                flag: "legacy-reservation-draft-tiered".to_string(),
                percent: 6.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 36876.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_reservation_draft_engine(&flags::new_registry());
        let item = new_legacy_reservation_draft("code-838", "label-200");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
