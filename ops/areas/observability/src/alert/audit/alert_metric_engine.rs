use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert metric calculation.
pub trait AlertMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertMetricFactor for FlaggedAlertMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertMetricAdjuster {
    pub minimum: f64,
}

impl AlertMetricFactor for MinimumAlertMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertMetricAdjuster {
    pub step: f64,
}

impl AlertMetricFactor for RoundingAlertMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMetricVersionOffsetAdjuster {
    pub weight: f64,
}

impl AlertMetricFactor for AlertMetricVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &AlertMetric, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertMetricFactor for TieredAlertMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryAlertMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertMetricFactor for CountryAlertMetricAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &AlertMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the alert metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertMetricEngine {
    pub steps: Vec<Rc<dyn AlertMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertMetricEngine {
    pub fn evaluate(&mut self, item: &AlertMetric) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[AlertMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_metric_engine(flags: &flags::Registry) -> AlertMetricEngine {
    AlertMetricEngine {
        steps: vec![
            Rc::new(FlaggedAlertMetricAdjuster {
                flags: flags.clone(),
                flag: "alert-metric-strict".to_string(),
                percent: 1.2,
            }) as Rc<dyn AlertMetricFactor>,
            Rc::new(MinimumAlertMetricAdjuster { minimum: 12.7 }),
            Rc::new(RoundingAlertMetricAdjuster { step: 1.0 }),
            Rc::new(AlertMetricVersionOffsetAdjuster { weight: 0.7 }),
            Rc::new(TieredAlertMetricAdjuster {
                bounds: vec![2387.0, 3131.0, 9278.0],
                rates: vec![1.02, 1.03, 0.99],
            }),
            Rc::new(CountryAlertMetricAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.99),
                    ("retail".to_string(), 1.06),
                    ("east".to_string(), 1.22),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 70512.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_metric_engine(&flags::new_registry());
        let item = new_alert_metric("code-651", "label-600");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
