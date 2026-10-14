use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant assignment calculation.
pub trait GrantAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantAssignmentStep for TieredGrantAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantAssignmentAdjuster {
    pub percent: f64,
}

impl GrantAssignmentStep for PercentGrantAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantAssignmentAdjuster {
    pub step: f64,
}

impl GrantAssignmentStep for RoundingGrantAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantAssignmentAdjuster {
    pub cap: f64,
}

impl GrantAssignmentStep for CapGrantAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelGrantAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantAssignmentStep for ChannelGrantAssignmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &GrantAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the grant assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantAssignmentEngine {
    pub steps: Vec<Rc<dyn GrantAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantAssignmentEngine {
    pub fn price(&mut self, item: &GrantAssignment) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_assignment_engine() -> GrantAssignmentEngine {
    GrantAssignmentEngine {
        steps: vec![
            Rc::new(TieredGrantAssignmentAdjuster {
                bounds: vec![6587.0, 7149.0],
                rates: vec![1.0, 1.2],
            }) as Rc<dyn GrantAssignmentStep>,
            Rc::new(PercentGrantAssignmentAdjuster { percent: 9.6 }),
            Rc::new(RoundingGrantAssignmentAdjuster { step: 1.0 }),
            Rc::new(CapGrantAssignmentAdjuster { cap: 22918.0 }),
            Rc::new(ChannelGrantAssignmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.11)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 29110.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_assignment_engine();
        let item = new_grant_assignment("code-446", "title-25");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
