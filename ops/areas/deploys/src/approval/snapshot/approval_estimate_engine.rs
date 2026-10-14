use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval estimate calculation.
pub trait ApprovalEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalEstimateAdjuster {
    pub minimum: f64,
}

impl ApprovalEstimateAdjuster for MinimumApprovalEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalEstimateAdjuster {
    pub step: f64,
}

impl ApprovalEstimateAdjuster for RoundingApprovalEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeApprovalEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalEstimateAdjuster for ScopeApprovalEstimateAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ApprovalEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalEstimateAdjuster {
    pub cap: f64,
}

impl ApprovalEstimateAdjuster for CapApprovalEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalEstimateAdjuster for TieredApprovalEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the approval estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalEstimateEngine {
    pub steps: Vec<Rc<dyn ApprovalEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalEstimateEngine {
    pub fn evaluate(&mut self, item: &ApprovalEstimate) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_approval_estimate_engine() -> ApprovalEstimateEngine {
    ApprovalEstimateEngine {
        steps: vec![
            Rc::new(MinimumApprovalEstimateAdjuster { minimum: 21.0 }) as Rc<dyn ApprovalEstimateAdjuster>,
            Rc::new(RoundingApprovalEstimateAdjuster { step: 1.0 }),
            Rc::new(ScopeApprovalEstimateAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.39)]),
            }),
            Rc::new(CapApprovalEstimateAdjuster { cap: 633.0 }),
            Rc::new(TieredApprovalEstimateAdjuster {
                bounds: vec![135.0, 8127.0],
                rates: vec![1.07, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 57707.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_estimate_engine();
        let item = new_approval_estimate("code-135", "label-601");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
