use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback approval entry calculation.
pub trait FallbackApprovalEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackApprovalEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackApprovalEntryAdjuster {
    pub percent: f64,
}

impl FallbackApprovalEntryFactor for PercentFallbackApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackApprovalEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackApprovalEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackApprovalEntryFactor for FlaggedFallbackApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackApprovalEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackApprovalEntryAdjuster {
    pub step: f64,
}

impl FallbackApprovalEntryFactor for RoundingFallbackApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackApprovalEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackApprovalEntryAdjuster {
    pub minimum: f64,
}

impl FallbackApprovalEntryFactor for MinimumFallbackApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackApprovalEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackApprovalEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackApprovalEntryFactor for TieredFallbackApprovalEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackApprovalEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback approval entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackApprovalEntryEngine {
    pub steps: Vec<Rc<dyn FallbackApprovalEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackApprovalEntryEngine {
    pub fn run(&mut self, item: &FallbackApprovalEntry) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackApprovalEntryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackApprovalEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_approval_entry_engine(flags: &flags::Registry) -> FallbackApprovalEntryEngine {
    FallbackApprovalEntryEngine {
        steps: vec![
            Rc::new(PercentFallbackApprovalEntryAdjuster { percent: 16.2 }) as Rc<dyn FallbackApprovalEntryFactor>,
            Rc::new(FlaggedFallbackApprovalEntryAdjuster {
                flags: flags.clone(),
                flag: "fallback-approval-entry-manual".to_string(),
                percent: -3.6,
            }),
            Rc::new(RoundingFallbackApprovalEntryAdjuster { step: 0.1 }),
            Rc::new(MinimumFallbackApprovalEntryAdjuster { minimum: 34.9 }),
            Rc::new(TieredFallbackApprovalEntryAdjuster {
                bounds: vec![1186.0, 6264.0],
                rates: vec![1.05, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 61232.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_approval_entry_engine(&flags::new_registry());
        let item = new_fallback_approval_entry("code-586", "label-852");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
