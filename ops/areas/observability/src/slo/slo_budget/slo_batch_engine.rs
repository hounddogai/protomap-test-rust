use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo batch calculation.
pub trait SloBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloBatchAdjuster {
    pub cap: f64,
}

impl SloBatchFactor for CapSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloBatchAdjuster {
    pub minimum: f64,
}

impl SloBatchFactor for MinimumSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSloBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloBatchFactor for SourceSloBatchAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SloBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloBatchAdjuster {
    pub step: f64,
}

impl SloBatchFactor for RoundingSloBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the slo batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloBatchEngine {
    pub steps: Vec<Rc<dyn SloBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloBatchEngine {
    pub fn run(&mut self, item: &SloBatch) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_slo_batch_engine() -> SloBatchEngine {
    SloBatchEngine {
        steps: vec![
            Rc::new(CapSloBatchAdjuster { cap: 35450.0 }) as Rc<dyn SloBatchFactor>,
            Rc::new(MinimumSloBatchAdjuster { minimum: 5.2 }),
            Rc::new(SourceSloBatchAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.09),
                    ("north".to_string(), 1.0),
                    ("west".to_string(), 1.27),
                ]),
            }),
            Rc::new(RoundingSloBatchAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 33502.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_batch_engine();
        let item = new_slo_batch("code-85", "title-608");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
