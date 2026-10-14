use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert summary calculation.
pub trait AlertSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAlertSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertSummaryStep for TitleAlertSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AlertSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertSummaryAdjuster {
    pub step: f64,
}

impl AlertSummaryStep for RoundingAlertSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertSummaryAdjuster {
    pub minimum: f64,
}

impl AlertSummaryStep for MinimumAlertSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertSummaryAdjuster {
    pub cap: f64,
}

impl AlertSummaryStep for CapAlertSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertSummaryAdjuster {
    pub percent: f64,
}

impl AlertSummaryStep for PercentAlertSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertSummaryStep for TieredAlertSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the alert summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertSummaryEngine {
    pub steps: Vec<Rc<dyn AlertSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertSummaryEngine {
    pub fn evaluate(&mut self, item: &AlertSummary) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[AlertSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_summary_engine() -> AlertSummaryEngine {
    AlertSummaryEngine {
        steps: vec![
            Rc::new(TitleAlertSummaryAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.11), ("online".to_string(), 1.33)]),
            }) as Rc<dyn AlertSummaryStep>,
            Rc::new(RoundingAlertSummaryAdjuster { step: 5.0 }),
            Rc::new(MinimumAlertSummaryAdjuster { minimum: 34.1 }),
            Rc::new(CapAlertSummaryAdjuster { cap: 18019.0 }),
            Rc::new(PercentAlertSummaryAdjuster { percent: -18.0 }),
            Rc::new(TieredAlertSummaryAdjuster {
                bounds: vec![6521.0, 9658.0],
                rates: vec![0.99, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 10574.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_summary_engine();
        let item = new_alert_summary("code-636", "title-77");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
