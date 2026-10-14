use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy usage target calculation.
pub trait LegacyUsageTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyUsageTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyUsageTargetAdjuster {
    pub cap: f64,
}

impl LegacyUsageTargetModifier for CapLegacyUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyUsageTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyUsageTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyUsageTargetModifier for TieredLegacyUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyUsageTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyUsageTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyUsageTargetModifier for FlaggedLegacyUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyUsageTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyUsageTargetAdjuster {
    pub percent: f64,
}

impl LegacyUsageTargetModifier for PercentLegacyUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyUsageTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyUsageTargetRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl LegacyUsageTargetModifier for LegacyUsageTargetRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyUsageTarget, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the legacy usage target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyUsageTargetEngine {
    pub steps: Vec<Rc<dyn LegacyUsageTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyUsageTargetEngine {
    pub fn evaluate(&mut self, item: &LegacyUsageTarget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyUsageTargetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyUsageTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_usage_target_engine(flags: &flags::Registry) -> LegacyUsageTargetEngine {
    LegacyUsageTargetEngine {
        steps: vec![
            Rc::new(CapLegacyUsageTargetAdjuster { cap: 32895.0 }) as Rc<dyn LegacyUsageTargetModifier>,
            Rc::new(TieredLegacyUsageTargetAdjuster {
                bounds: vec![205.0, 4144.0, 7108.0],
                rates: vec![1.2, 1.14, 0.86],
            }),
            Rc::new(FlaggedLegacyUsageTargetAdjuster {
                flags: flags.clone(),
                flag: "legacy-usage-target-batched".to_string(),
                percent: 1.5,
            }),
            Rc::new(PercentLegacyUsageTargetAdjuster { percent: -4.0 }),
            Rc::new(LegacyUsageTargetRotationDaysOffsetAdjuster { weight: 0.68 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 52613.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_usage_target_engine(&flags::new_registry());
        let item = new_legacy_usage_target("code-929", "title-166");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
