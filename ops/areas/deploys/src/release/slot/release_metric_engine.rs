use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release metric calculation.
pub trait ReleaseMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseMetricAdjuster {
    pub cap: f64,
}

impl ReleaseMetricStep for CapReleaseMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseMetricAdjuster {
    pub minimum: f64,
}

impl ReleaseMetricStep for MinimumReleaseMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseMetricStep for FlaggedReleaseMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseMetricAdjuster {
    pub percent: f64,
}

impl ReleaseMetricStep for PercentReleaseMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMetricThresholdOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseMetricStep for ReleaseMetricThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseMetric, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseMetricAdjuster {
    pub step: f64,
}

impl ReleaseMetricStep for RoundingReleaseMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the release metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseMetricEngine {
    pub steps: Vec<Rc<dyn ReleaseMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseMetricEngine {
    pub fn run(&mut self, item: &ReleaseMetric) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_metric_engine(flags: &flags::Registry) -> ReleaseMetricEngine {
    ReleaseMetricEngine {
        steps: vec![
            Rc::new(CapReleaseMetricAdjuster { cap: 43955.0 }) as Rc<dyn ReleaseMetricStep>,
            Rc::new(MinimumReleaseMetricAdjuster { minimum: 16.9 }),
            Rc::new(FlaggedReleaseMetricAdjuster {
                flags: flags.clone(),
                flag: "release-metric-default".to_string(),
                percent: 0.2,
            }),
            Rc::new(PercentReleaseMetricAdjuster { percent: -0.3 }),
            Rc::new(ReleaseMetricThresholdOffsetAdjuster { weight: 1.9 }),
            Rc::new(RoundingReleaseMetricAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 74049.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_metric_engine(&flags::new_registry());
        let item = new_release_metric("code-985", "label-844");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
