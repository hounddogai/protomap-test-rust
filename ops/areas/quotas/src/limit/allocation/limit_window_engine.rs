use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit window calculation.
pub trait LimitWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitWindowAdjuster {
    pub cap: f64,
}

impl LimitWindowModifier for CapLimitWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitWindowModifier for FlaggedLimitWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindowBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl LimitWindowModifier for LimitWindowBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &LimitWindow, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitWindowAdjuster {
    pub minimum: f64,
}

impl LimitWindowModifier for MinimumLimitWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitWindowModifier for TieredLimitWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitWindowAdjuster {
    pub percent: f64,
}

impl LimitWindowModifier for PercentLimitWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the limit window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitWindowEngine {
    pub steps: Vec<Rc<dyn LimitWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitWindowEngine {
    pub fn price(&mut self, item: &LimitWindow) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LimitWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_limit_window_engine(flags: &flags::Registry) -> LimitWindowEngine {
    LimitWindowEngine {
        steps: vec![
            Rc::new(CapLimitWindowAdjuster { cap: 31561.0 }) as Rc<dyn LimitWindowModifier>,
            Rc::new(FlaggedLimitWindowAdjuster {
                flags: flags.clone(),
                flag: "limit-window-regional".to_string(),
                percent: 0.5,
            }),
            Rc::new(LimitWindowBatchSizeOffsetAdjuster { weight: 0.3 }),
            Rc::new(MinimumLimitWindowAdjuster { minimum: 26.4 }),
            Rc::new(TieredLimitWindowAdjuster {
                bounds: vec![474.0, 3371.0],
                rates: vec![0.87, 1.03],
            }),
            Rc::new(PercentLimitWindowAdjuster { percent: -6.1 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 49014.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_window_engine(&flags::new_registry());
        let item = new_limit_window("code-848", "name-796");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
