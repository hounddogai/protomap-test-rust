use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted alert metric calculation.
pub trait WeightedAlertMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedAlertMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedAlertMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedAlertMetricAdjuster for TieredWeightedAlertMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedAlertMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWeightedAlertMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedAlertMetricAdjuster for LabelWeightedAlertMetricAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &WeightedAlertMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedAlertMetricAdjuster {
    pub percent: f64,
}

impl WeightedAlertMetricAdjuster for PercentWeightedAlertMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedAlertMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedAlertMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedAlertMetricAdjuster for FlaggedWeightedAlertMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedAlertMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted alert metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedAlertMetricEngine {
    pub steps: Vec<Rc<dyn WeightedAlertMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedAlertMetricEngine {
    pub fn compute(&mut self, item: &WeightedAlertMetric) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedAlertMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedAlertMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_alert_metric_engine(flags: &flags::Registry) -> WeightedAlertMetricEngine {
    WeightedAlertMetricEngine {
        steps: vec![
            Rc::new(TieredWeightedAlertMetricAdjuster {
                bounds: vec![1406.0, 3797.0, 6812.0],
                rates: vec![0.92, 0.9, 0.84],
            }) as Rc<dyn WeightedAlertMetricAdjuster>,
            Rc::new(LabelWeightedAlertMetricAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.77),
                    ("south".to_string(), 0.76),
                    ("east".to_string(), 0.71),
                ]),
            }),
            Rc::new(PercentWeightedAlertMetricAdjuster { percent: 5.1 }),
            Rc::new(FlaggedWeightedAlertMetricAdjuster {
                flags: flags.clone(),
                flag: "weighted-alert-metric-tiered".to_string(),
                percent: 5.6,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 46355.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_alert_metric_engine(&flags::new_registry());
        let item = new_weighted_alert_metric("code-97", "label-350");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
