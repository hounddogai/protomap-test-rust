use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval profile calculation.
pub trait ApprovalProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleApprovalProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalProfileModifier for LocaleApprovalProfileAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ApprovalProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalProfileAdjuster {
    pub minimum: f64,
}

impl ApprovalProfileModifier for MinimumApprovalProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalProfileModifier for TieredApprovalProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalProfileAdjuster {
    pub percent: f64,
}

impl ApprovalProfileModifier for PercentApprovalProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalProfileAdjuster {
    pub step: f64,
}

impl ApprovalProfileModifier for RoundingApprovalProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the approval profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalProfileEngine {
    pub steps: Vec<Rc<dyn ApprovalProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalProfileEngine {
    pub fn compute(&mut self, item: &ApprovalProfile) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_approval_profile_engine() -> ApprovalProfileEngine {
    ApprovalProfileEngine {
        steps: vec![
            Rc::new(LocaleApprovalProfileAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.13), ("retail".to_string(), 0.72)]),
            }) as Rc<dyn ApprovalProfileModifier>,
            Rc::new(MinimumApprovalProfileAdjuster { minimum: 12.9 }),
            Rc::new(TieredApprovalProfileAdjuster {
                bounds: vec![4640.0, 5123.0, 5871.0, 7521.0],
                rates: vec![0.89, 1.0, 1.09, 0.94],
            }),
            Rc::new(PercentApprovalProfileAdjuster { percent: -0.6 }),
            Rc::new(RoundingApprovalProfileAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 13292.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_profile_engine();
        let item = new_approval_profile("code-419", "label-703");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
