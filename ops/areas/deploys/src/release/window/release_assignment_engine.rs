use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release assignment calculation.
pub trait ReleaseAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameReleaseAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseAssignmentModifier for NameReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ReleaseAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseAssignmentModifier for TieredReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseAssignmentAdjuster {
    pub step: f64,
}

impl ReleaseAssignmentModifier for RoundingReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseAssignmentModifier for FlaggedReleaseAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseAssignmentEngine {
    pub steps: Vec<Rc<dyn ReleaseAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseAssignmentEngine {
    pub fn price(&mut self, item: &ReleaseAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_release_assignment_engine(flags: &flags::Registry) -> ReleaseAssignmentEngine {
    ReleaseAssignmentEngine {
        steps: vec![
            Rc::new(NameReleaseAssignmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.38)]),
            }) as Rc<dyn ReleaseAssignmentModifier>,
            Rc::new(TieredReleaseAssignmentAdjuster {
                bounds: vec![1588.0, 2555.0],
                rates: vec![1.18, 1.08],
            }),
            Rc::new(RoundingReleaseAssignmentAdjuster { step: 0.05 }),
            Rc::new(FlaggedReleaseAssignmentAdjuster {
                flags: flags.clone(),
                flag: "release-assignment-primary".to_string(),
                percent: -2.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 32701.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_assignment_engine(&flags::new_registry());
        let item = new_release_assignment("code-615", "name-755");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
