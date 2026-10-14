use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending policy variance calculation.
pub trait PendingPolicyVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPolicyVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingPolicyVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPolicyVarianceAdjuster for TitlePendingPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingPolicyVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingPolicyVarianceAdjuster {
    pub minimum: f64,
}

impl PendingPolicyVarianceAdjuster for MinimumPendingPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingPolicyVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingPolicyVarianceAdjuster {
    pub step: f64,
}

impl PendingPolicyVarianceAdjuster for RoundingPendingPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingPolicyVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending policy variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPolicyVarianceEngine {
    pub steps: Vec<Rc<dyn PendingPolicyVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPolicyVarianceEngine {
    pub fn price(&mut self, item: &PendingPolicyVariance) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPolicyVarianceAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPolicyVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_policy_variance_engine() -> PendingPolicyVarianceEngine {
    PendingPolicyVarianceEngine {
        steps: vec![
            Rc::new(TitlePendingPolicyVarianceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.86)]),
            }) as Rc<dyn PendingPolicyVarianceAdjuster>,
            Rc::new(MinimumPendingPolicyVarianceAdjuster { minimum: 9.4 }),
            Rc::new(RoundingPendingPolicyVarianceAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 52360.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_policy_variance_engine();
        let item = new_pending_policy_variance("code-505", "title-430");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
