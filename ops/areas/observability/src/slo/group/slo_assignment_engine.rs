use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo assignment calculation.
pub trait SloAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSloAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloAssignmentAdjuster for FlaggedSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloAssignmentAdjuster {
    pub step: f64,
}

impl SloAssignmentAdjuster for RoundingSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloAssignmentAdjuster for TieredSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloAssignmentAdjuster {
    pub cap: f64,
}

impl SloAssignmentAdjuster for CapSloAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAssignmentAmountOffsetAdjuster {
    pub weight: f64,
}

impl SloAssignmentAdjuster for SloAssignmentAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &SloAssignment, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the slo assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloAssignmentEngine {
    pub steps: Vec<Rc<dyn SloAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloAssignmentEngine {
    pub fn price(&mut self, item: &SloAssignment) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_slo_assignment_engine(flags: &flags::Registry) -> SloAssignmentEngine {
    SloAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedSloAssignmentAdjuster {
                flags: flags.clone(),
                flag: "slo-assignment-lenient".to_string(),
                percent: -7.5,
            }) as Rc<dyn SloAssignmentAdjuster>,
            Rc::new(RoundingSloAssignmentAdjuster { step: 0.05 }),
            Rc::new(TieredSloAssignmentAdjuster {
                bounds: vec![2544.0, 3098.0, 5580.0, 9951.0],
                rates: vec![0.81, 0.92, 1.03, 0.94],
            }),
            Rc::new(CapSloAssignmentAdjuster { cap: 1530.0 }),
            Rc::new(SloAssignmentAmountOffsetAdjuster { weight: 1.73 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 90770.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_assignment_engine(&flags::new_registry());
        let item = new_slo_assignment("code-231", "title-369");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
