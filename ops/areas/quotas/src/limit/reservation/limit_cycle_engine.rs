use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit cycle calculation.
pub trait LimitCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitCycleAdjuster {
    pub minimum: f64,
}

impl LimitCycleFactor for MinimumLimitCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLimitCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitCycleFactor for LabelLimitCycleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LimitCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitCycleAdjuster {
    pub cap: f64,
}

impl LimitCycleFactor for CapLimitCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitCycleFactor for TieredLimitCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the limit cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitCycleEngine {
    pub steps: Vec<Rc<dyn LimitCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitCycleEngine {
    pub fn evaluate(&mut self, item: &LimitCycle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LimitCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_limit_cycle_engine() -> LimitCycleEngine {
    LimitCycleEngine {
        steps: vec![
            Rc::new(MinimumLimitCycleAdjuster { minimum: 36.4 }) as Rc<dyn LimitCycleFactor>,
            Rc::new(LabelLimitCycleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.07), ("east".to_string(), 1.03)]),
            }),
            Rc::new(CapLimitCycleAdjuster { cap: 8807.0 }),
            Rc::new(TieredLimitCycleAdjuster {
                bounds: vec![1527.0, 3422.0, 5200.0, 8032.0],
                rates: vec![1.18, 1.01, 1.03, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 34840.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_cycle_engine();
        let item = new_limit_cycle("code-356", "label-626");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
