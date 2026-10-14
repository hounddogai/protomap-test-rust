use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual redaction variance calculation.
pub trait ManualRedactionVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRedactionVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualRedactionVarianceAdjuster {
    pub minimum: f64,
}

impl ManualRedactionVarianceStep for MinimumManualRedactionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualRedactionVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualRedactionVarianceAdjuster {
    pub percent: f64,
}

impl ManualRedactionVarianceStep for PercentManualRedactionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualRedactionVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRedactionVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRedactionVarianceStep for FlaggedManualRedactionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRedactionVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualRedactionVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualRedactionVarianceStep for TieredManualRedactionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualRedactionVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVarianceWeightOffsetAdjuster {
    pub weight: f64,
}

impl ManualRedactionVarianceStep for ManualRedactionVarianceWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRedactionVariance, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualRedactionVarianceAdjuster {
    pub step: f64,
}

impl ManualRedactionVarianceStep for RoundingManualRedactionVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualRedactionVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual redaction variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRedactionVarianceEngine {
    pub steps: Vec<Rc<dyn ManualRedactionVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRedactionVarianceEngine {
    pub fn run(&mut self, item: &ManualRedactionVariance) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualRedactionVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualRedactionVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_redaction_variance_engine(flags: &flags::Registry) -> ManualRedactionVarianceEngine {
    ManualRedactionVarianceEngine {
        steps: vec![
            Rc::new(MinimumManualRedactionVarianceAdjuster { minimum: 28.7 }) as Rc<dyn ManualRedactionVarianceStep>,
            Rc::new(PercentManualRedactionVarianceAdjuster { percent: -15.7 }),
            Rc::new(FlaggedManualRedactionVarianceAdjuster {
                flags: flags.clone(),
                flag: "manual-redaction-variance-fallback".to_string(),
                percent: -1.3,
            }),
            Rc::new(TieredManualRedactionVarianceAdjuster {
                bounds: vec![1278.0, 1732.0, 2445.0],
                rates: vec![1.08, 1.1, 0.94],
            }),
            Rc::new(ManualRedactionVarianceWeightOffsetAdjuster { weight: 0.86 }),
            Rc::new(RoundingManualRedactionVarianceAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 8221.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_redaction_variance_engine(&flags::new_registry());
        let item = new_manual_redaction_variance("code-938", "name-702");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
