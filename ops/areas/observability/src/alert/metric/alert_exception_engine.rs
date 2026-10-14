use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert exception calculation.
pub trait AlertExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertExceptionAdjuster {
    pub step: f64,
}

impl AlertExceptionStep for RoundingAlertExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAlertExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertExceptionStep for LabelAlertExceptionAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AlertException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertExceptionRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl AlertExceptionStep for AlertExceptionRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &AlertException, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertExceptionStep for TieredAlertExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the alert exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertExceptionEngine {
    pub steps: Vec<Rc<dyn AlertExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertExceptionEngine {
    pub fn evaluate(&mut self, item: &AlertException) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[AlertException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_exception_engine() -> AlertExceptionEngine {
    AlertExceptionEngine {
        steps: vec![
            Rc::new(RoundingAlertExceptionAdjuster { step: 1.0 }) as Rc<dyn AlertExceptionStep>,
            Rc::new(LabelAlertExceptionAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.33)]),
            }),
            Rc::new(AlertExceptionRotationDaysOffsetAdjuster { weight: 1.5 }),
            Rc::new(TieredAlertExceptionAdjuster {
                bounds: vec![74.0, 6863.0, 8212.0],
                rates: vec![1.15, 0.86, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 38077.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_exception_engine();
        let item = new_alert_exception("code-879", "label-816");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
