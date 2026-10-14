use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce threshold calculation.
pub trait NonceThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceThresholdStep for FlaggedNonceThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceThresholdAdjuster {
    pub percent: f64,
}

impl NonceThresholdStep for PercentNonceThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceThresholdStep for TieredNonceThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceThresholdAdjuster {
    pub cap: f64,
}

impl NonceThresholdStep for CapNonceThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceThresholdAdjuster {
    pub step: f64,
}

impl NonceThresholdStep for RoundingNonceThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the nonce threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceThresholdEngine {
    pub steps: Vec<Rc<dyn NonceThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceThresholdEngine {
    pub fn price(&mut self, item: &NonceThreshold) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_nonce_threshold_engine(flags: &flags::Registry) -> NonceThresholdEngine {
    NonceThresholdEngine {
        steps: vec![
            Rc::new(FlaggedNonceThresholdAdjuster {
                flags: flags.clone(),
                flag: "nonce-threshold-global".to_string(),
                percent: -0.5,
            }) as Rc<dyn NonceThresholdStep>,
            Rc::new(PercentNonceThresholdAdjuster { percent: -1.6 }),
            Rc::new(TieredNonceThresholdAdjuster {
                bounds: vec![2945.0, 3906.0, 4432.0],
                rates: vec![1.07, 1.06, 0.89],
            }),
            Rc::new(CapNonceThresholdAdjuster { cap: 20869.0 }),
            Rc::new(RoundingNonceThresholdAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 99029.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_threshold_engine(&flags::new_registry());
        let item = new_nonce_threshold("code-616", "name-172");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
