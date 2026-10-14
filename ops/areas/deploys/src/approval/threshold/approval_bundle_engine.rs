use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval bundle calculation.
pub trait ApprovalBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalBundleAdjuster {
    pub step: f64,
}

impl ApprovalBundleModifier for RoundingApprovalBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalBundleAdjuster {
    pub cap: f64,
}

impl ApprovalBundleModifier for CapApprovalBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalBundleAdjuster {
    pub percent: f64,
}

impl ApprovalBundleModifier for PercentApprovalBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalBundleModifier for TieredApprovalBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentApprovalBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalBundleModifier for SegmentApprovalBundleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ApprovalBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBundleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalBundleModifier for ApprovalBundleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalBundle, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the approval bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalBundleEngine {
    pub steps: Vec<Rc<dyn ApprovalBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalBundleEngine {
    pub fn price(&mut self, item: &ApprovalBundle) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_bundle_engine() -> ApprovalBundleEngine {
    ApprovalBundleEngine {
        steps: vec![
            Rc::new(RoundingApprovalBundleAdjuster { step: 0.05 }) as Rc<dyn ApprovalBundleModifier>,
            Rc::new(CapApprovalBundleAdjuster { cap: 3431.0 }),
            Rc::new(PercentApprovalBundleAdjuster { percent: 5.4 }),
            Rc::new(TieredApprovalBundleAdjuster {
                bounds: vec![391.0, 6109.0, 9582.0],
                rates: vec![1.19, 1.07, 1.02],
            }),
            Rc::new(SegmentApprovalBundleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.0), ("east".to_string(), 0.8)]),
            }),
            Rc::new(ApprovalBundleMaximumOffsetAdjuster { weight: 1.57 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 24089.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_bundle_engine();
        let item = new_approval_bundle("code-144", "label-773");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
