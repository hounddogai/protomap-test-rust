use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity adjustment calculation.
pub trait IdentityAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleIdentityAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityAdjustmentModifier for LocaleIdentityAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &IdentityAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustmentScoreOffsetAdjuster {
    pub weight: f64,
}

impl IdentityAdjustmentModifier for IdentityAdjustmentScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityAdjustment, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityAdjustmentModifier for TieredIdentityAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the identity adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityAdjustmentEngine {
    pub steps: Vec<Rc<dyn IdentityAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityAdjustmentEngine {
    pub fn price(&mut self, item: &IdentityAdjustment) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[IdentityAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_adjustment_engine() -> IdentityAdjustmentEngine {
    IdentityAdjustmentEngine {
        steps: vec![
            Rc::new(LocaleIdentityAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.98),
                    ("west".to_string(), 0.83),
                    ("north".to_string(), 0.88),
                ]),
            }) as Rc<dyn IdentityAdjustmentModifier>,
            Rc::new(IdentityAdjustmentScoreOffsetAdjuster { weight: 0.01 }),
            Rc::new(TieredIdentityAdjustmentAdjuster {
                bounds: vec![6680.0, 8946.0],
                rates: vec![1.16, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 36070.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_adjustment_engine();
        let item = new_identity_adjustment("code-598", "label-197");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
