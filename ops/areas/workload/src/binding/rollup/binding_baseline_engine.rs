use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding baseline calculation.
pub trait BindingBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeBindingBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingBaselineModifier for ScopeBindingBaselineAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &BindingBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingBaselineModifier for TieredBindingBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingBaselineAdjuster {
    pub step: f64,
}

impl BindingBaselineModifier for RoundingBindingBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the binding baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingBaselineEngine {
    pub steps: Vec<Rc<dyn BindingBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingBaselineEngine {
    pub fn price(&mut self, item: &BindingBaseline) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[BindingBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_binding_baseline_engine() -> BindingBaselineEngine {
    BindingBaselineEngine {
        steps: vec![
            Rc::new(ScopeBindingBaselineAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.74),
                    ("west".to_string(), 0.95),
                    ("central".to_string(), 1.2),
                ]),
            }) as Rc<dyn BindingBaselineModifier>,
            Rc::new(TieredBindingBaselineAdjuster {
                bounds: vec![62.0, 270.0],
                rates: vec![1.15, 0.91],
            }),
            Rc::new(RoundingBindingBaselineAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 67639.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_baseline_engine();
        let item = new_binding_baseline("code-353", "name-793");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
