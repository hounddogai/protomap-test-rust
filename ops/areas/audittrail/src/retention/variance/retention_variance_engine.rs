use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention variance calculation.
pub trait RetentionVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionVarianceAdjuster {
    pub step: f64,
}

impl RetentionVarianceFactor for RoundingRetentionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionVarianceFactor for FlaggedRetentionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionVarianceFactor for TieredRetentionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionVarianceAdjuster {
    pub cap: f64,
}

impl RetentionVarianceFactor for CapRetentionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionVarianceAdjuster {
    pub minimum: f64,
}

impl RetentionVarianceFactor for MinimumRetentionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the retention variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionVarianceEngine {
    pub steps: Vec<Rc<dyn RetentionVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionVarianceEngine {
    pub fn run(&mut self, item: &RetentionVariance) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RetentionVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_variance_engine(flags: &flags::Registry) -> RetentionVarianceEngine {
    RetentionVarianceEngine {
        steps: vec![
            Rc::new(RoundingRetentionVarianceAdjuster { step: 1.0 }) as Rc<dyn RetentionVarianceFactor>,
            Rc::new(FlaggedRetentionVarianceAdjuster {
                flags: flags.clone(),
                flag: "retention-variance-adaptive".to_string(),
                percent: 6.6,
            }),
            Rc::new(TieredRetentionVarianceAdjuster {
                bounds: vec![423.0, 6645.0, 7532.0],
                rates: vec![0.88, 0.97, 0.85],
            }),
            Rc::new(CapRetentionVarianceAdjuster { cap: 25556.0 }),
            Rc::new(MinimumRetentionVarianceAdjuster { minimum: 29.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 63211.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_variance_engine(&flags::new_registry());
        let item = new_retention_variance("code-239", "label-689");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
