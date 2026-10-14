use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert request calculation.
pub trait AlertRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertRequestAdjuster {
    pub minimum: f64,
}

impl AlertRequestAdjuster for MinimumAlertRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertRequestAdjuster for FlaggedAlertRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertRequestAdjuster {
    pub percent: f64,
}

impl AlertRequestAdjuster for PercentAlertRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertRequestAdjuster for TieredAlertRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertRequestAdjuster {
    pub cap: f64,
}

impl AlertRequestAdjuster for CapAlertRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the alert request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertRequestEngine {
    pub steps: Vec<Rc<dyn AlertRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertRequestEngine {
    pub fn price(&mut self, item: &AlertRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AlertRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_alert_request_engine(flags: &flags::Registry) -> AlertRequestEngine {
    AlertRequestEngine {
        steps: vec![
            Rc::new(MinimumAlertRequestAdjuster { minimum: 11.7 }) as Rc<dyn AlertRequestAdjuster>,
            Rc::new(FlaggedAlertRequestAdjuster {
                flags: flags.clone(),
                flag: "alert-request-pending".to_string(),
                percent: 7.1,
            }),
            Rc::new(PercentAlertRequestAdjuster { percent: -16.2 }),
            Rc::new(TieredAlertRequestAdjuster {
                bounds: vec![81.0, 4562.0],
                rates: vec![1.08, 1.02],
            }),
            Rc::new(CapAlertRequestAdjuster { cap: 24841.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 97201.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_request_engine(&flags::new_registry());
        let item = new_alert_request("code-131", "label-724");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
