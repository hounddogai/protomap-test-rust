use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback approval digest calculation.
pub trait FallbackApprovalDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackApprovalDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalDigestMinimumOffsetAdjuster {
    pub weight: f64,
}

impl FallbackApprovalDigestModifier for FallbackApprovalDigestMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackApprovalDigest, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackApprovalDigestAdjuster {
    pub percent: f64,
}

impl FallbackApprovalDigestModifier for PercentFallbackApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackApprovalDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackApprovalDigestAdjuster {
    pub cap: f64,
}

impl FallbackApprovalDigestModifier for CapFallbackApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackApprovalDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackApprovalDigestAdjuster {
    pub step: f64,
}

impl FallbackApprovalDigestModifier for RoundingFallbackApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackApprovalDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackApprovalDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackApprovalDigestModifier for FlaggedFallbackApprovalDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackApprovalDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFallbackApprovalDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackApprovalDigestModifier for CategoryFallbackApprovalDigestAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &FallbackApprovalDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback approval digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackApprovalDigestEngine {
    pub steps: Vec<Rc<dyn FallbackApprovalDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackApprovalDigestEngine {
    pub fn price(&mut self, item: &FallbackApprovalDigest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackApprovalDigestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackApprovalDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_approval_digest_engine(flags: &flags::Registry) -> FallbackApprovalDigestEngine {
    FallbackApprovalDigestEngine {
        steps: vec![
            Rc::new(FallbackApprovalDigestMinimumOffsetAdjuster { weight: 0.09 })
                as Rc<dyn FallbackApprovalDigestModifier>,
            Rc::new(PercentFallbackApprovalDigestAdjuster { percent: 24.0 }),
            Rc::new(CapFallbackApprovalDigestAdjuster { cap: 41763.0 }),
            Rc::new(RoundingFallbackApprovalDigestAdjuster { step: 5.0 }),
            Rc::new(FlaggedFallbackApprovalDigestAdjuster {
                flags: flags.clone(),
                flag: "fallback-approval-digest-regional".to_string(),
                percent: -8.3,
            }),
            Rc::new(CategoryFallbackApprovalDigestAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.96),
                    ("retail".to_string(), 1.14),
                    ("north".to_string(), 0.86),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 54553.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_approval_digest_engine(&flags::new_registry());
        let item = new_fallback_approval_digest("code-810", "label-40");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
