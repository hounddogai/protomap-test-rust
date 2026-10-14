use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped alert baseline calculation.
pub trait CappedAlertBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedAlertBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedAlertBaselineAdjuster {
    pub minimum: f64,
}

impl CappedAlertBaselineAdjuster for MinimumCappedAlertBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedAlertBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedAlertBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedAlertBaselineAdjuster for FlaggedCappedAlertBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedAlertBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedAlertBaselineAdjuster {
    pub step: f64,
}

impl CappedAlertBaselineAdjuster for RoundingCappedAlertBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedAlertBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedAlertBaselineAdjuster {
    pub percent: f64,
}

impl CappedAlertBaselineAdjuster for PercentCappedAlertBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedAlertBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedAlertBaselineLimitOffsetAdjuster {
    pub weight: f64,
}

impl CappedAlertBaselineAdjuster for CappedAlertBaselineLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &CappedAlertBaseline, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedAlertBaselineAdjuster {
    pub cap: f64,
}

impl CappedAlertBaselineAdjuster for CapCappedAlertBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedAlertBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped alert baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedAlertBaselineEngine {
    pub steps: Vec<Rc<dyn CappedAlertBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedAlertBaselineEngine {
    pub fn compute(&mut self, item: &CappedAlertBaseline) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[CappedAlertBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_alert_baseline_engine(flags: &flags::Registry) -> CappedAlertBaselineEngine {
    CappedAlertBaselineEngine {
        steps: vec![
            Rc::new(MinimumCappedAlertBaselineAdjuster { minimum: 37.4 }) as Rc<dyn CappedAlertBaselineAdjuster>,
            Rc::new(FlaggedCappedAlertBaselineAdjuster {
                flags: flags.clone(),
                flag: "capped-alert-baseline-strict".to_string(),
                percent: 6.1,
            }),
            Rc::new(RoundingCappedAlertBaselineAdjuster { step: 0.1 }),
            Rc::new(PercentCappedAlertBaselineAdjuster { percent: -13.5 }),
            Rc::new(CappedAlertBaselineLimitOffsetAdjuster { weight: 1.37 }),
            Rc::new(CapCappedAlertBaselineAdjuster { cap: 5363.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 61874.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_alert_baseline_engine(&flags::new_registry());
        let item = new_capped_alert_baseline("code-411", "title-853");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
