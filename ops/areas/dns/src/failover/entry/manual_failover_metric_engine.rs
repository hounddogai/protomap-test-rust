use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual failover metric calculation.
pub trait ManualFailoverMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualFailoverMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualFailoverMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualFailoverMetricFactor for FlaggedManualFailoverMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualFailoverMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualFailoverMetricAdjuster {
    pub percent: f64,
}

impl ManualFailoverMetricFactor for PercentManualFailoverMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualFailoverMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualFailoverMetricAdjuster {
    pub minimum: f64,
}

impl ManualFailoverMetricFactor for MinimumManualFailoverMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualFailoverMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualFailoverMetricAdjuster {
    pub cap: f64,
}

impl ManualFailoverMetricFactor for CapManualFailoverMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualFailoverMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceManualFailoverMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualFailoverMetricFactor for SourceManualFailoverMetricAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ManualFailoverMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverMetricBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl ManualFailoverMetricFactor for ManualFailoverMetricBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &ManualFailoverMetric, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

/// Runs the manual failover metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualFailoverMetricEngine {
    pub steps: Vec<Rc<dyn ManualFailoverMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualFailoverMetricEngine {
    pub fn run(&mut self, item: &ManualFailoverMetric) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualFailoverMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualFailoverMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_failover_metric_engine(flags: &flags::Registry) -> ManualFailoverMetricEngine {
    ManualFailoverMetricEngine {
        steps: vec![
            Rc::new(FlaggedManualFailoverMetricAdjuster {
                flags: flags.clone(),
                flag: "manual-failover-metric-primary".to_string(),
                percent: 9.6,
            }) as Rc<dyn ManualFailoverMetricFactor>,
            Rc::new(PercentManualFailoverMetricAdjuster { percent: -11.0 }),
            Rc::new(MinimumManualFailoverMetricAdjuster { minimum: 32.7 }),
            Rc::new(CapManualFailoverMetricAdjuster { cap: 3874.0 }),
            Rc::new(SourceManualFailoverMetricAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.06),
                    ("east".to_string(), 0.78),
                    ("central".to_string(), 1.31),
                ]),
            }),
            Rc::new(ManualFailoverMetricBatchSizeOffsetAdjuster { weight: 0.73 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 92093.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_failover_metric_engine(&flags::new_registry());
        let item = new_manual_failover_metric("code-42", "title-57");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
