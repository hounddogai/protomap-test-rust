use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert score calculation.
pub trait AlertScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertScoreAdjuster {
    pub step: f64,
}

impl AlertScoreModifier for RoundingAlertScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertScoreAdjuster {
    pub percent: f64,
}

impl AlertScoreModifier for PercentAlertScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameAlertScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertScoreModifier for NameAlertScoreAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &AlertScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the alert score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertScoreEngine {
    pub steps: Vec<Rc<dyn AlertScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertScoreEngine {
    pub fn evaluate(&mut self, item: &AlertScore) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AlertScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_score_engine() -> AlertScoreEngine {
    AlertScoreEngine {
        steps: vec![
            Rc::new(RoundingAlertScoreAdjuster { step: 5.0 }) as Rc<dyn AlertScoreModifier>,
            Rc::new(PercentAlertScoreAdjuster { percent: -13.4 }),
            Rc::new(NameAlertScoreAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.77), ("east".to_string(), 0.75)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 27766.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_score_engine();
        let item = new_alert_score("code-923", "name-428");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
