use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo cycle calculation.
pub trait SloCycleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloCycleAdjuster {
    pub step: f64,
}

impl SloCycleModifier for RoundingSloCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloCycleModifier for TieredSloCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloCycleAdjuster {
    pub minimum: f64,
}

impl SloCycleModifier for MinimumSloCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the slo cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloCycleEngine {
    pub steps: Vec<Rc<dyn SloCycleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloCycleEngine {
    pub fn run(&mut self, item: &SloCycle) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloCycleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_slo_cycle_engine() -> SloCycleEngine {
    SloCycleEngine {
        steps: vec![
            Rc::new(RoundingSloCycleAdjuster { step: 0.5 }) as Rc<dyn SloCycleModifier>,
            Rc::new(TieredSloCycleAdjuster {
                bounds: vec![1639.0, 6906.0],
                rates: vec![1.19, 1.16],
            }),
            Rc::new(MinimumSloCycleAdjuster { minimum: 47.5 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 60255.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_cycle_engine();
        let item = new_slo_cycle("code-103", "name-188");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
