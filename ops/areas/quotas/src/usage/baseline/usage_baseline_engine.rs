use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the usage baseline calculation.
pub trait UsageBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageBaselineFactor for FlaggedUsageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBaselineScoreOffsetAdjuster {
    pub weight: f64,
}

impl UsageBaselineFactor for UsageBaselineScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &UsageBaseline, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageBaselineAdjuster {
    pub cap: f64,
}

impl UsageBaselineFactor for CapUsageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageBaselineAdjuster {
    pub minimum: f64,
}

impl UsageBaselineFactor for MinimumUsageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageBaselineAdjuster {
    pub step: f64,
}

impl UsageBaselineFactor for RoundingUsageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the usage baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageBaselineEngine {
    pub steps: Vec<Rc<dyn UsageBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageBaselineEngine {
    pub fn price(&mut self, item: &UsageBaseline) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[UsageBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_usage_baseline_engine(flags: &flags::Registry) -> UsageBaselineEngine {
    UsageBaselineEngine {
        steps: vec![
            Rc::new(FlaggedUsageBaselineAdjuster {
                flags: flags.clone(),
                flag: "usage-baseline-scheduled".to_string(),
                percent: 0.1,
            }) as Rc<dyn UsageBaselineFactor>,
            Rc::new(UsageBaselineScoreOffsetAdjuster { weight: 0.72 }),
            Rc::new(CapUsageBaselineAdjuster { cap: 48722.0 }),
            Rc::new(MinimumUsageBaselineAdjuster { minimum: 36.8 }),
            Rc::new(RoundingUsageBaselineAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 76252.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_baseline_engine(&flags::new_registry());
        let item = new_usage_baseline("code-537", "name-156");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
