use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict approval baseline calculation.
pub trait StrictApprovalBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictApprovalBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictApprovalBaselineAdjuster {
    pub cap: f64,
}

impl StrictApprovalBaselineModifier for CapStrictApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictApprovalBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictApprovalBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictApprovalBaselineModifier for TieredStrictApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictApprovalBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictApprovalBaselineAdjuster {
    pub step: f64,
}

impl StrictApprovalBaselineModifier for RoundingStrictApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictApprovalBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalBaselineQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl StrictApprovalBaselineModifier for StrictApprovalBaselineQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &StrictApprovalBaseline, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictApprovalBaselineAdjuster {
    pub percent: f64,
}

impl StrictApprovalBaselineModifier for PercentStrictApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictApprovalBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionStrictApprovalBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictApprovalBaselineModifier for RegionStrictApprovalBaselineAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &StrictApprovalBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict approval baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictApprovalBaselineEngine {
    pub steps: Vec<Rc<dyn StrictApprovalBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictApprovalBaselineEngine {
    pub fn run(&mut self, item: &StrictApprovalBaseline) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictApprovalBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictApprovalBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_approval_baseline_engine() -> StrictApprovalBaselineEngine {
    StrictApprovalBaselineEngine {
        steps: vec![
            Rc::new(CapStrictApprovalBaselineAdjuster { cap: 44162.0 }) as Rc<dyn StrictApprovalBaselineModifier>,
            Rc::new(TieredStrictApprovalBaselineAdjuster {
                bounds: vec![2230.0, 3152.0, 3230.0, 6807.0],
                rates: vec![0.85, 1.18, 0.99, 0.95],
            }),
            Rc::new(RoundingStrictApprovalBaselineAdjuster { step: 0.25 }),
            Rc::new(StrictApprovalBaselineQuotaUnitsOffsetAdjuster { weight: 1.84 }),
            Rc::new(PercentStrictApprovalBaselineAdjuster { percent: -8.1 }),
            Rc::new(RegionStrictApprovalBaselineAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.36),
                    ("west".to_string(), 1.11),
                    ("central".to_string(), 1.21),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 25761.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_approval_baseline_engine();
        let item = new_strict_approval_baseline("code-383", "label-895");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
