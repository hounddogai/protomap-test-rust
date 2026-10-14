use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback draft calculation.
pub trait RollbackDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRollbackDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackDraftAdjuster for ClusterRollbackDraftAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RollbackDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackDraftAdjuster {
    pub minimum: f64,
}

impl RollbackDraftAdjuster for MinimumRollbackDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackDraftAdjuster {
    pub cap: f64,
}

impl RollbackDraftAdjuster for CapRollbackDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackDraftAdjuster for TieredRollbackDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackDraftAdjuster {
    pub step: f64,
}

impl RollbackDraftAdjuster for RoundingRollbackDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackDraftAdjuster {
    pub percent: f64,
}

impl RollbackDraftAdjuster for PercentRollbackDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackDraftEngine {
    pub steps: Vec<Rc<dyn RollbackDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackDraftEngine {
    pub fn price(&mut self, item: &RollbackDraft) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackDraftAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_draft_engine() -> RollbackDraftEngine {
    RollbackDraftEngine {
        steps: vec![
            Rc::new(ClusterRollbackDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.09), ("central".to_string(), 0.95)]),
            }) as Rc<dyn RollbackDraftAdjuster>,
            Rc::new(MinimumRollbackDraftAdjuster { minimum: 49.4 }),
            Rc::new(CapRollbackDraftAdjuster { cap: 27623.0 }),
            Rc::new(TieredRollbackDraftAdjuster {
                bounds: vec![2630.0, 3431.0, 3473.0, 8913.0],
                rates: vec![0.95, 1.1, 1.04, 0.84],
            }),
            Rc::new(RoundingRollbackDraftAdjuster { step: 5.0 }),
            Rc::new(PercentRollbackDraftAdjuster { percent: -8.4 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 69218.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_draft_engine();
        let item = new_rollback_draft("code-992", "name-744");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
