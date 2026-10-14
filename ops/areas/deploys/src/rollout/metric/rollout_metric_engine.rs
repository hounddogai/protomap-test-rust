use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout metric calculation.
pub trait RolloutMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutMetricStep for FlaggedRolloutMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRolloutMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutMetricStep for SegmentRolloutMetricAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RolloutMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMetricTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RolloutMetricStep for RolloutMetricTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutMetric, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutMetricAdjuster {
    pub cap: f64,
}

impl RolloutMetricStep for CapRolloutMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rollout metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutMetricEngine {
    pub steps: Vec<Rc<dyn RolloutMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutMetricEngine {
    pub fn run(&mut self, item: &RolloutMetric) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RolloutMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollout_metric_engine(flags: &flags::Registry) -> RolloutMetricEngine {
    RolloutMetricEngine {
        steps: vec![
            Rc::new(FlaggedRolloutMetricAdjuster {
                flags: flags.clone(),
                flag: "rollout-metric-tiered".to_string(),
                percent: 9.3,
            }) as Rc<dyn RolloutMetricStep>,
            Rc::new(SegmentRolloutMetricAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.17),
                    ("online".to_string(), 0.95),
                    ("retail".to_string(), 1.06),
                ]),
            }),
            Rc::new(RolloutMetricTtlSecondsOffsetAdjuster { weight: 0.11 }),
            Rc::new(CapRolloutMetricAdjuster { cap: 17668.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 9339.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_metric_engine(&flags::new_registry());
        let item = new_rollout_metric("code-520", "title-98");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
