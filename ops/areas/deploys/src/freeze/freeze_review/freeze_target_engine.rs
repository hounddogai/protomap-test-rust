use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze target calculation.
pub trait FreezeTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezeTargetAdjuster {
    pub step: f64,
}

impl FreezeTargetFactor for RoundingFreezeTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezeTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFreezeTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeTargetFactor for NameFreezeTargetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FreezeTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTargetThresholdOffsetAdjuster {
    pub weight: f64,
}

impl FreezeTargetFactor for FreezeTargetThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeTarget, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeTargetAdjuster {
    pub minimum: f64,
}

impl FreezeTargetFactor for MinimumFreezeTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeTargetAdjuster {
    pub cap: f64,
}

impl FreezeTargetFactor for CapFreezeTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the freeze target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeTargetEngine {
    pub steps: Vec<Rc<dyn FreezeTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeTargetEngine {
    pub fn run(&mut self, item: &FreezeTarget) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FreezeTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_target_engine() -> FreezeTargetEngine {
    FreezeTargetEngine {
        steps: vec![
            Rc::new(RoundingFreezeTargetAdjuster { step: 0.25 }) as Rc<dyn FreezeTargetFactor>,
            Rc::new(NameFreezeTargetAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.18),
                    ("online".to_string(), 1.36),
                    ("retail".to_string(), 1.18),
                ]),
            }),
            Rc::new(FreezeTargetThresholdOffsetAdjuster { weight: 0.2 }),
            Rc::new(MinimumFreezeTargetAdjuster { minimum: 25.3 }),
            Rc::new(CapFreezeTargetAdjuster { cap: 2989.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 75691.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_target_engine();
        let item = new_freeze_target("code-794", "name-253");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
