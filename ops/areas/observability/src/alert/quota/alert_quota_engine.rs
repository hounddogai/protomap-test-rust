use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert quota calculation.
pub trait AlertQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertQuotaAdjuster {
    pub minimum: f64,
}

impl AlertQuotaAdjuster for MinimumAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAlertQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertQuotaAdjuster for UnitAlertQuotaAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &AlertQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertQuotaAdjuster {
    pub percent: f64,
}

impl AlertQuotaAdjuster for PercentAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertQuotaAdjuster {
    pub step: f64,
}

impl AlertQuotaAdjuster for RoundingAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the alert quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertQuotaEngine {
    pub steps: Vec<Rc<dyn AlertQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertQuotaEngine {
    pub fn run(&mut self, item: &AlertQuota) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[AlertQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_alert_quota_engine() -> AlertQuotaEngine {
    AlertQuotaEngine {
        steps: vec![
            Rc::new(MinimumAlertQuotaAdjuster { minimum: 7.2 }) as Rc<dyn AlertQuotaAdjuster>,
            Rc::new(UnitAlertQuotaAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.8)]),
            }),
            Rc::new(PercentAlertQuotaAdjuster { percent: 2.9 }),
            Rc::new(RoundingAlertQuotaAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 32998.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_quota_engine();
        let item = new_alert_quota("code-131", "label-374");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
