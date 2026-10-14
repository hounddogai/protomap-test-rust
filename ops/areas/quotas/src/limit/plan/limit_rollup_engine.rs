use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit rollup calculation.
pub trait LimitRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRollupThresholdOffsetAdjuster {
    pub weight: f64,
}

impl LimitRollupAdjuster for LimitRollupThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &LimitRollup, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitRollupAdjuster for FlaggedLimitRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitRollupAdjuster {
    pub minimum: f64,
}

impl LimitRollupAdjuster for MinimumLimitRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitRollupAdjuster {
    pub cap: f64,
}

impl LimitRollupAdjuster for CapLimitRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitRollupAdjuster for TieredLimitRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the limit rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitRollupEngine {
    pub steps: Vec<Rc<dyn LimitRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitRollupEngine {
    pub fn evaluate(&mut self, item: &LimitRollup) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LimitRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_limit_rollup_engine(flags: &flags::Registry) -> LimitRollupEngine {
    LimitRollupEngine {
        steps: vec![
            Rc::new(LimitRollupThresholdOffsetAdjuster { weight: 1.43 }) as Rc<dyn LimitRollupAdjuster>,
            Rc::new(FlaggedLimitRollupAdjuster {
                flags: flags.clone(),
                flag: "limit-rollup-strict".to_string(),
                percent: -6.1,
            }),
            Rc::new(MinimumLimitRollupAdjuster { minimum: 12.2 }),
            Rc::new(CapLimitRollupAdjuster { cap: 30768.0 }),
            Rc::new(TieredLimitRollupAdjuster {
                bounds: vec![1049.0, 6563.0, 7652.0, 7960.0],
                rates: vec![0.9, 1.11, 1.15, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 26658.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_rollup_engine(&flags::new_registry());
        let item = new_limit_rollup("code-598", "label-671");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
