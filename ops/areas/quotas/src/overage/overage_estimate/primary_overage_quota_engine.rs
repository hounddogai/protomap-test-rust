use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary overage quota calculation.
pub trait PrimaryOverageQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryOverageQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaDiscountOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryOverageQuotaStep for PrimaryOverageQuotaDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryOverageQuota, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryOverageQuotaAdjuster {
    pub step: f64,
}

impl PrimaryOverageQuotaStep for RoundingPrimaryOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryOverageQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryOverageQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryOverageQuotaStep for FlaggedPrimaryOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryOverageQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryOverageQuotaAdjuster {
    pub percent: f64,
}

impl PrimaryOverageQuotaStep for PercentPrimaryOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryOverageQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryOverageQuotaAdjuster {
    pub cap: f64,
}

impl PrimaryOverageQuotaStep for CapPrimaryOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryOverageQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary overage quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryOverageQuotaEngine {
    pub steps: Vec<Rc<dyn PrimaryOverageQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryOverageQuotaEngine {
    pub fn compute(&mut self, item: &PrimaryOverageQuota) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryOverageQuotaStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryOverageQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_overage_quota_engine(flags: &flags::Registry) -> PrimaryOverageQuotaEngine {
    PrimaryOverageQuotaEngine {
        steps: vec![
            Rc::new(PrimaryOverageQuotaDiscountOffsetAdjuster { weight: 0.77 }) as Rc<dyn PrimaryOverageQuotaStep>,
            Rc::new(RoundingPrimaryOverageQuotaAdjuster { step: 0.5 }),
            Rc::new(FlaggedPrimaryOverageQuotaAdjuster {
                flags: flags.clone(),
                flag: "primary-overage-quota-global".to_string(),
                percent: 4.2,
            }),
            Rc::new(PercentPrimaryOverageQuotaAdjuster { percent: -2.9 }),
            Rc::new(CapPrimaryOverageQuotaAdjuster { cap: 32599.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 85613.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_overage_quota_engine(&flags::new_registry());
        let item = new_primary_overage_quota("code-447", "label-345");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
