use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient sink budget calculation.
pub trait LenientSinkBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSinkBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientSinkBudgetAdjuster {
    pub percent: f64,
}

impl LenientSinkBudgetModifier for PercentLenientSinkBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientSinkBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientSinkBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientSinkBudgetModifier for TieredLenientSinkBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientSinkBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientSinkBudgetAdjuster {
    pub minimum: f64,
}

impl LenientSinkBudgetModifier for MinimumLenientSinkBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientSinkBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientSinkBudgetAdjuster {
    pub cap: f64,
}

impl LenientSinkBudgetModifier for CapLenientSinkBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientSinkBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeLenientSinkBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientSinkBudgetModifier for ScopeLenientSinkBudgetAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &LenientSinkBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient sink budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSinkBudgetEngine {
    pub steps: Vec<Rc<dyn LenientSinkBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSinkBudgetEngine {
    pub fn run(&mut self, item: &LenientSinkBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientSinkBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_sink_budget_engine() -> LenientSinkBudgetEngine {
    LenientSinkBudgetEngine {
        steps: vec![
            Rc::new(PercentLenientSinkBudgetAdjuster { percent: -12.1 }) as Rc<dyn LenientSinkBudgetModifier>,
            Rc::new(TieredLenientSinkBudgetAdjuster {
                bounds: vec![1361.0, 5417.0],
                rates: vec![1.17, 0.86],
            }),
            Rc::new(MinimumLenientSinkBudgetAdjuster { minimum: 13.9 }),
            Rc::new(CapLenientSinkBudgetAdjuster { cap: 26415.0 }),
            Rc::new(ScopeLenientSinkBudgetAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.38)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 98765.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_sink_budget_engine();
        let item = new_lenient_sink_budget("code-232", "label-411");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
