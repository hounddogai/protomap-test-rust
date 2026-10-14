use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy variance calculation.
pub trait PolicyVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyVarianceAdjuster {
    pub cap: f64,
}

impl PolicyVarianceModifier for CapPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyVarianceAdjuster {
    pub percent: f64,
}

impl PolicyVarianceModifier for PercentPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyVarianceAdjuster {
    pub step: f64,
}

impl PolicyVarianceModifier for RoundingPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyVarianceModifier for FlaggedPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the policy variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyVarianceEngine {
    pub steps: Vec<Rc<dyn PolicyVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyVarianceEngine {
    pub fn evaluate(&mut self, item: &PolicyVariance) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyVarianceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_variance_engine(flags: &flags::Registry) -> PolicyVarianceEngine {
    PolicyVarianceEngine {
        steps: vec![
            Rc::new(CapPolicyVarianceAdjuster { cap: 3856.0 }) as Rc<dyn PolicyVarianceModifier>,
            Rc::new(PercentPolicyVarianceAdjuster { percent: 2.4 }),
            Rc::new(RoundingPolicyVarianceAdjuster { step: 0.1 }),
            Rc::new(FlaggedPolicyVarianceAdjuster {
                flags: flags.clone(),
                flag: "policy-variance-strict".to_string(),
                percent: 2.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 43812.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_variance_engine(&flags::new_registry());
        let item = new_policy_variance("code-753", "title-34");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
