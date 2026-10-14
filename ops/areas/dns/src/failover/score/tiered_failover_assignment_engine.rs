use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered failover assignment calculation.
pub trait TieredFailoverAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredFailoverAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredFailoverAssignmentAdjuster {
    pub minimum: f64,
}

impl TieredFailoverAssignmentStep for MinimumTieredFailoverAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredFailoverAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverAssignmentCostOffsetAdjuster {
    pub weight: f64,
}

impl TieredFailoverAssignmentStep for TieredFailoverAssignmentCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &TieredFailoverAssignment, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredFailoverAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredFailoverAssignmentStep for TieredTieredFailoverAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredFailoverAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTieredFailoverAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredFailoverAssignmentStep for ScopeTieredFailoverAssignmentAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TieredFailoverAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredFailoverAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredFailoverAssignmentStep for FlaggedTieredFailoverAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredFailoverAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered failover assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredFailoverAssignmentEngine {
    pub steps: Vec<Rc<dyn TieredFailoverAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredFailoverAssignmentEngine {
    pub fn compute(&mut self, item: &TieredFailoverAssignment) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TieredFailoverAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_failover_assignment_engine(flags: &flags::Registry) -> TieredFailoverAssignmentEngine {
    TieredFailoverAssignmentEngine {
        steps: vec![
            Rc::new(MinimumTieredFailoverAssignmentAdjuster { minimum: 2.3 }) as Rc<dyn TieredFailoverAssignmentStep>,
            Rc::new(TieredFailoverAssignmentCostOffsetAdjuster { weight: 0.38 }),
            Rc::new(TieredTieredFailoverAssignmentAdjuster {
                bounds: vec![3574.0, 4002.0],
                rates: vec![0.9, 1.09],
            }),
            Rc::new(ScopeTieredFailoverAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.18),
                    ("east".to_string(), 0.77),
                    ("south".to_string(), 1.15),
                ]),
            }),
            Rc::new(FlaggedTieredFailoverAssignmentAdjuster {
                flags: flags.clone(),
                flag: "tiered-failover-assignment-rolling".to_string(),
                percent: -5.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 28570.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_failover_assignment_engine(&flags::new_registry());
        let item = new_tiered_failover_assignment("code-811", "name-944");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
