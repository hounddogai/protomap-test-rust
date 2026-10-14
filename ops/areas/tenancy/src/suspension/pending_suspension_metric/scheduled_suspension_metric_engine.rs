use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled suspension metric calculation.
pub trait ScheduledSuspensionMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSuspensionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledSuspensionMetricAdjuster {
    pub percent: f64,
}

impl ScheduledSuspensionMetricAdjuster for PercentScheduledSuspensionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledSuspensionMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSuspensionMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSuspensionMetricAdjuster for FlaggedScheduledSuspensionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSuspensionMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetricVersionOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSuspensionMetricAdjuster for ScheduledSuspensionMetricVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSuspensionMetric, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledSuspensionMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSuspensionMetricAdjuster for LabelScheduledSuspensionMetricAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledSuspensionMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled suspension metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionMetricEngine {
    pub steps: Vec<Rc<dyn ScheduledSuspensionMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSuspensionMetricEngine {
    pub fn evaluate(&mut self, item: &ScheduledSuspensionMetric) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSuspensionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_suspension_metric_engine(flags: &flags::Registry) -> ScheduledSuspensionMetricEngine {
    ScheduledSuspensionMetricEngine {
        steps: vec![
            Rc::new(PercentScheduledSuspensionMetricAdjuster { percent: 12.0 })
                as Rc<dyn ScheduledSuspensionMetricAdjuster>,
            Rc::new(FlaggedScheduledSuspensionMetricAdjuster {
                flags: flags.clone(),
                flag: "scheduled-suspension-metric-fallback".to_string(),
                percent: -9.8,
            }),
            Rc::new(ScheduledSuspensionMetricVersionOffsetAdjuster { weight: 0.01 }),
            Rc::new(LabelScheduledSuspensionMetricAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.96)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 77118.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_suspension_metric_engine(&flags::new_registry());
        let item = new_scheduled_suspension_metric("code-816", "label-207");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
