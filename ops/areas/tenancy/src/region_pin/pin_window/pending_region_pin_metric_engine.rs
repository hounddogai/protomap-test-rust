use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending region pin metric calculation.
pub trait PendingRegionPinMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRegionPinMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePendingRegionPinMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRegionPinMetricAdjuster for LocalePendingRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PendingRegionPinMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinMetricCostOffsetAdjuster {
    pub weight: f64,
}

impl PendingRegionPinMetricAdjuster for PendingRegionPinMetricCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &PendingRegionPinMetric, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingRegionPinMetricAdjuster {
    pub cap: f64,
}

impl PendingRegionPinMetricAdjuster for CapPendingRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingRegionPinMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRegionPinMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRegionPinMetricAdjuster for FlaggedPendingRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRegionPinMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRegionPinMetricAdjuster {
    pub percent: f64,
}

impl PendingRegionPinMetricAdjuster for PercentPendingRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRegionPinMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRegionPinMetricAdjuster {
    pub minimum: f64,
}

impl PendingRegionPinMetricAdjuster for MinimumPendingRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRegionPinMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending region pin metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRegionPinMetricEngine {
    pub steps: Vec<Rc<dyn PendingRegionPinMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRegionPinMetricEngine {
    pub fn run(&mut self, item: &PendingRegionPinMetric) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingRegionPinMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_region_pin_metric_engine(flags: &flags::Registry) -> PendingRegionPinMetricEngine {
    PendingRegionPinMetricEngine {
        steps: vec![
            Rc::new(LocalePendingRegionPinMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.16)]),
            }) as Rc<dyn PendingRegionPinMetricAdjuster>,
            Rc::new(PendingRegionPinMetricCostOffsetAdjuster { weight: 1.94 }),
            Rc::new(CapPendingRegionPinMetricAdjuster { cap: 6164.0 }),
            Rc::new(FlaggedPendingRegionPinMetricAdjuster {
                flags: flags.clone(),
                flag: "pending-region-pin-metric-global".to_string(),
                percent: 3.1,
            }),
            Rc::new(PercentPendingRegionPinMetricAdjuster { percent: 11.8 }),
            Rc::new(MinimumPendingRegionPinMetricAdjuster { minimum: 48.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 20807.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_region_pin_metric_engine(&flags::new_registry());
        let item = new_pending_region_pin_metric("code-159", "name-484");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
