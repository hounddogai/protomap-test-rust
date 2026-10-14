use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert rollup calculation.
pub trait AlertRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertRollupAdjuster {
    pub cap: f64,
}

impl AlertRollupModifier for CapAlertRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertRollupAdjuster {
    pub step: f64,
}

impl AlertRollupModifier for RoundingAlertRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAlertRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertRollupModifier for TitleAlertRollupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AlertRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRollupEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl AlertRollupModifier for AlertRollupEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &AlertRollup, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertRollupModifier for TieredAlertRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the alert rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertRollupEngine {
    pub steps: Vec<Rc<dyn AlertRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertRollupEngine {
    pub fn price(&mut self, item: &AlertRollup) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_alert_rollup_engine() -> AlertRollupEngine {
    AlertRollupEngine {
        steps: vec![
            Rc::new(CapAlertRollupAdjuster { cap: 6455.0 }) as Rc<dyn AlertRollupModifier>,
            Rc::new(RoundingAlertRollupAdjuster { step: 0.25 }),
            Rc::new(TitleAlertRollupAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.34),
                    ("online".to_string(), 0.88),
                    ("north".to_string(), 0.77),
                ]),
            }),
            Rc::new(AlertRollupEndsAtOffsetAdjuster { weight: 0.6 }),
            Rc::new(TieredAlertRollupAdjuster {
                bounds: vec![4507.0, 4739.0, 7246.0, 9501.0],
                rates: vec![0.82, 0.84, 0.98, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 3820.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_rollup_engine();
        let item = new_alert_rollup("code-930", "title-328");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
