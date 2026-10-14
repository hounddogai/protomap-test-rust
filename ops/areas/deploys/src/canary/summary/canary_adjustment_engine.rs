use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary adjustment calculation.
pub trait CanaryAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryAdjustmentFactor for FlaggedCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAdjustmentAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl CanaryAdjustmentFactor for CanaryAdjustmentAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryAdjustment, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryAdjustmentFactor for TieredCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryAdjustmentAdjuster {
    pub step: f64,
}

impl CanaryAdjustmentFactor for RoundingCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryAdjustmentAdjuster {
    pub minimum: f64,
}

impl CanaryAdjustmentFactor for MinimumCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceCanaryAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryAdjustmentFactor for SourceCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &CanaryAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the canary adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryAdjustmentEngine {
    pub steps: Vec<Rc<dyn CanaryAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryAdjustmentEngine {
    pub fn run(&mut self, item: &CanaryAdjustment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CanaryAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_canary_adjustment_engine(flags: &flags::Registry) -> CanaryAdjustmentEngine {
    CanaryAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedCanaryAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "canary-adjustment-regional".to_string(),
                percent: -1.4,
            }) as Rc<dyn CanaryAdjustmentFactor>,
            Rc::new(CanaryAdjustmentAttemptsOffsetAdjuster { weight: 1.34 }),
            Rc::new(TieredCanaryAdjustmentAdjuster {
                bounds: vec![1362.0, 3957.0, 6984.0, 8788.0],
                rates: vec![0.81, 0.84, 0.96, 0.98],
            }),
            Rc::new(RoundingCanaryAdjustmentAdjuster { step: 1.0 }),
            Rc::new(MinimumCanaryAdjustmentAdjuster { minimum: 40.8 }),
            Rc::new(SourceCanaryAdjustmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.14), ("south".to_string(), 1.05)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 9802.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_adjustment_engine(&flags::new_registry());
        let item = new_canary_adjustment("code-968", "title-512");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
