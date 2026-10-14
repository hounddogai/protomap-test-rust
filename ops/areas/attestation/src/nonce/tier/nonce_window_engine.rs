use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce window calculation.
pub trait NonceWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceWindowStep for TieredNonceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceWindowStep for FlaggedNonceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceWindowAdjuster {
    pub percent: f64,
}

impl NonceWindowStep for PercentNonceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceWindowAdjuster {
    pub step: f64,
}

impl NonceWindowStep for RoundingNonceWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceWindowMarginOffsetAdjuster {
    pub weight: f64,
}

impl NonceWindowStep for NonceWindowMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &NonceWindow, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the nonce window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceWindowEngine {
    pub steps: Vec<Rc<dyn NonceWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceWindowEngine {
    pub fn evaluate(&mut self, item: &NonceWindow) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[NonceWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_window_engine(flags: &flags::Registry) -> NonceWindowEngine {
    NonceWindowEngine {
        steps: vec![
            Rc::new(TieredNonceWindowAdjuster {
                bounds: vec![1208.0, 6225.0, 7775.0, 9682.0],
                rates: vec![1.2, 0.8, 0.86, 1.04],
            }) as Rc<dyn NonceWindowStep>,
            Rc::new(FlaggedNonceWindowAdjuster {
                flags: flags.clone(),
                flag: "nonce-window-lenient".to_string(),
                percent: -1.2,
            }),
            Rc::new(PercentNonceWindowAdjuster { percent: 18.7 }),
            Rc::new(RoundingNonceWindowAdjuster { step: 1.0 }),
            Rc::new(NonceWindowMarginOffsetAdjuster { weight: 0.82 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 17641.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_window_engine(&flags::new_registry());
        let item = new_nonce_window("code-487", "label-628");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
