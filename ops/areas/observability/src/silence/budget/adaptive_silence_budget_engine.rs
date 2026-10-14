use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive silence budget calculation.
pub trait AdaptiveSilenceBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveSilenceBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceBudgetDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveSilenceBudgetAdjuster for AdaptiveSilenceBudgetDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveSilenceBudget, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveSilenceBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveSilenceBudgetAdjuster for TieredAdaptiveSilenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveSilenceBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveSilenceBudgetAdjuster {
    pub minimum: f64,
}

impl AdaptiveSilenceBudgetAdjuster for MinimumAdaptiveSilenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveSilenceBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveSilenceBudgetAdjuster {
    pub cap: f64,
}

impl AdaptiveSilenceBudgetAdjuster for CapAdaptiveSilenceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveSilenceBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive silence budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveSilenceBudgetEngine {
    pub steps: Vec<Rc<dyn AdaptiveSilenceBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveSilenceBudgetEngine {
    pub fn evaluate(&mut self, item: &AdaptiveSilenceBudget) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveSilenceBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_silence_budget_engine() -> AdaptiveSilenceBudgetEngine {
    AdaptiveSilenceBudgetEngine {
        steps: vec![
            Rc::new(AdaptiveSilenceBudgetDurationMinutesOffsetAdjuster { weight: 0.84 })
                as Rc<dyn AdaptiveSilenceBudgetAdjuster>,
            Rc::new(TieredAdaptiveSilenceBudgetAdjuster {
                bounds: vec![2040.0, 9178.0],
                rates: vec![0.87, 1.04],
            }),
            Rc::new(MinimumAdaptiveSilenceBudgetAdjuster { minimum: 37.2 }),
            Rc::new(CapAdaptiveSilenceBudgetAdjuster { cap: 25674.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 16657.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_silence_budget_engine();
        let item = new_adaptive_silence_budget("code-260", "label-873");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
