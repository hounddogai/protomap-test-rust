use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled usage estimate calculation.
pub trait ScheduledUsageEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledUsageEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledUsageEstimateAdjuster {
    pub cap: f64,
}

impl ScheduledUsageEstimateAdjuster for CapScheduledUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledUsageEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledUsageEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledUsageEstimateAdjuster for FlaggedScheduledUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledUsageEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledUsageEstimateAdjuster {
    pub minimum: f64,
}

impl ScheduledUsageEstimateAdjuster for MinimumScheduledUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledUsageEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledUsageEstimateAdjuster {
    pub percent: f64,
}

impl ScheduledUsageEstimateAdjuster for PercentScheduledUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledUsageEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledUsageEstimateAdjuster {
    pub step: f64,
}

impl ScheduledUsageEstimateAdjuster for RoundingScheduledUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledUsageEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledUsageEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledUsageEstimateAdjuster for TieredScheduledUsageEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledUsageEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the scheduled usage estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledUsageEstimateEngine {
    pub steps: Vec<Rc<dyn ScheduledUsageEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledUsageEstimateEngine {
    pub fn evaluate(&mut self, item: &ScheduledUsageEstimate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledUsageEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_usage_estimate_engine(flags: &flags::Registry) -> ScheduledUsageEstimateEngine {
    ScheduledUsageEstimateEngine {
        steps: vec![
            Rc::new(CapScheduledUsageEstimateAdjuster { cap: 4349.0 }) as Rc<dyn ScheduledUsageEstimateAdjuster>,
            Rc::new(FlaggedScheduledUsageEstimateAdjuster {
                flags: flags.clone(),
                flag: "scheduled-usage-estimate-regional".to_string(),
                percent: 7.7,
            }),
            Rc::new(MinimumScheduledUsageEstimateAdjuster { minimum: 1.5 }),
            Rc::new(PercentScheduledUsageEstimateAdjuster { percent: -6.1 }),
            Rc::new(RoundingScheduledUsageEstimateAdjuster { step: 5.0 }),
            Rc::new(TieredScheduledUsageEstimateAdjuster {
                bounds: vec![18.0, 1233.0, 2279.0, 5714.0],
                rates: vec![0.96, 1.08, 1.07, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 58114.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_usage_estimate_engine(&flags::new_registry());
        let item = new_scheduled_usage_estimate("code-222", "name-295");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
