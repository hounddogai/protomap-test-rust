use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict page target calculation.
pub trait StrictPageTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPageTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictPageTargetAdjuster {
    pub cap: f64,
}

impl StrictPageTargetAdjuster for CapStrictPageTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictPageTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictPageTargetAdjuster {
    pub minimum: f64,
}

impl StrictPageTargetAdjuster for MinimumStrictPageTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictPageTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictPageTargetAdjuster {
    pub step: f64,
}

impl StrictPageTargetAdjuster for RoundingStrictPageTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictPageTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictPageTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictPageTargetAdjuster for CategoryStrictPageTargetAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictPageTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict page target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPageTargetEngine {
    pub steps: Vec<Rc<dyn StrictPageTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPageTargetEngine {
    pub fn evaluate(&mut self, item: &StrictPageTarget) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictPageTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictPageTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_page_target_engine() -> StrictPageTargetEngine {
    StrictPageTargetEngine {
        steps: vec![
            Rc::new(CapStrictPageTargetAdjuster { cap: 17192.0 }) as Rc<dyn StrictPageTargetAdjuster>,
            Rc::new(MinimumStrictPageTargetAdjuster { minimum: 31.6 }),
            Rc::new(RoundingStrictPageTargetAdjuster { step: 0.1 }),
            Rc::new(CategoryStrictPageTargetAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.91),
                    ("central".to_string(), 0.98),
                    ("online".to_string(), 0.84),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 73525.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_page_target_engine();
        let item = new_strict_page_target("code-710", "title-723");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
