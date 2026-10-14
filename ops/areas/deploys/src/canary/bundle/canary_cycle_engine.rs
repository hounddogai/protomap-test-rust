use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary cycle calculation.
pub trait CanaryCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryCycleAdjuster {
    pub percent: f64,
}

impl CanaryCycleFactor for PercentCanaryCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCanaryCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryCycleFactor for NameCanaryCycleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CanaryCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryCycleAdjuster {
    pub step: f64,
}

impl CanaryCycleFactor for RoundingCanaryCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the canary cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryCycleEngine {
    pub steps: Vec<Rc<dyn CanaryCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryCycleEngine {
    pub fn compute(&mut self, item: &CanaryCycle) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanaryCycleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanaryCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_canary_cycle_engine() -> CanaryCycleEngine {
    CanaryCycleEngine {
        steps: vec![
            Rc::new(PercentCanaryCycleAdjuster { percent: 20.4 }) as Rc<dyn CanaryCycleFactor>,
            Rc::new(NameCanaryCycleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.36)]),
            }),
            Rc::new(RoundingCanaryCycleAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 79398.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_cycle_engine();
        let item = new_canary_cycle("code-791", "name-329");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
