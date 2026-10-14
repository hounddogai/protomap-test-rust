use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped forecast policy calculation.
pub trait CappedForecastPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedForecastPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitCappedForecastPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedForecastPolicyModifier for UnitCappedForecastPolicyAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &CappedForecastPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedForecastPolicyAdjuster {
    pub cap: f64,
}

impl CappedForecastPolicyModifier for CapCappedForecastPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedForecastPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastPolicyBurstOffsetAdjuster {
    pub weight: f64,
}

impl CappedForecastPolicyModifier for CappedForecastPolicyBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &CappedForecastPolicy, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the capped forecast policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedForecastPolicyEngine {
    pub steps: Vec<Rc<dyn CappedForecastPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedForecastPolicyEngine {
    pub fn price(&mut self, item: &CappedForecastPolicy) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedForecastPolicyModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedForecastPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_forecast_policy_engine() -> CappedForecastPolicyEngine {
    CappedForecastPolicyEngine {
        steps: vec![
            Rc::new(UnitCappedForecastPolicyAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.33)]),
            }) as Rc<dyn CappedForecastPolicyModifier>,
            Rc::new(CapCappedForecastPolicyAdjuster { cap: 12618.0 }),
            Rc::new(CappedForecastPolicyBurstOffsetAdjuster { weight: 0.42 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 72194.0,
    }
}
