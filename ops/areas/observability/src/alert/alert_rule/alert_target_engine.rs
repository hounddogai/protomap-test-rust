use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert target calculation.
pub trait AlertTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertTargetTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl AlertTargetAdjuster for AlertTargetTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &AlertTarget, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertTargetAdjuster {
    pub minimum: f64,
}

impl AlertTargetAdjuster for MinimumAlertTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertTargetAdjuster {
    pub cap: f64,
}

impl AlertTargetAdjuster for CapAlertTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertTargetAdjuster for TieredAlertTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertTargetAdjuster {
    pub step: f64,
}

impl AlertTargetAdjuster for RoundingAlertTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the alert target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertTargetEngine {
    pub steps: Vec<Rc<dyn AlertTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertTargetEngine {
    pub fn evaluate(&mut self, item: &AlertTarget) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_target_engine() -> AlertTargetEngine {
    AlertTargetEngine {
        steps: vec![
            Rc::new(AlertTargetTtlSecondsOffsetAdjuster { weight: 0.03 }) as Rc<dyn AlertTargetAdjuster>,
            Rc::new(MinimumAlertTargetAdjuster { minimum: 1.2 }),
            Rc::new(CapAlertTargetAdjuster { cap: 20638.0 }),
            Rc::new(TieredAlertTargetAdjuster {
                bounds: vec![1661.0, 5374.0, 7770.0],
                rates: vec![0.94, 0.95, 0.87],
            }),
            Rc::new(RoundingAlertTargetAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 99834.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_target_engine();
        let item = new_alert_target("code-167", "label-856");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
