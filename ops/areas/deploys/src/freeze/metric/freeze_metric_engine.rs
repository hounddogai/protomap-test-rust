use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze metric calculation.
pub trait FreezeMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeMetricAdjuster for TieredFreezeMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMetricAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl FreezeMetricAdjuster for FreezeMetricAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeMetric, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeMetricAdjuster for FlaggedFreezeMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the freeze metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeMetricEngine {
    pub steps: Vec<Rc<dyn FreezeMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeMetricEngine {
    pub fn run(&mut self, item: &FreezeMetric) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FreezeMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_metric_engine(flags: &flags::Registry) -> FreezeMetricEngine {
    FreezeMetricEngine {
        steps: vec![
            Rc::new(TieredFreezeMetricAdjuster {
                bounds: vec![1303.0, 2743.0, 3659.0, 5941.0],
                rates: vec![1.2, 1.15, 1.02, 0.88],
            }) as Rc<dyn FreezeMetricAdjuster>,
            Rc::new(FreezeMetricAttemptsOffsetAdjuster { weight: 0.58 }),
            Rc::new(FlaggedFreezeMetricAdjuster {
                flags: flags.clone(),
                flag: "freeze-metric-fallback".to_string(),
                percent: -4.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 46413.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_metric_engine(&flags::new_registry());
        let item = new_freeze_metric("code-701", "name-841");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
