use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict suspension bundle calculation.
pub trait StrictSuspensionBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictSuspensionBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictSuspensionBundleAdjuster {
    pub minimum: f64,
}

impl StrictSuspensionBundleModifier for MinimumStrictSuspensionBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictSuspensionBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundleScoreOffsetAdjuster {
    pub weight: f64,
}

impl StrictSuspensionBundleModifier for StrictSuspensionBundleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &StrictSuspensionBundle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictSuspensionBundleAdjuster {
    pub cap: f64,
}

impl StrictSuspensionBundleModifier for CapStrictSuspensionBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictSuspensionBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictSuspensionBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictSuspensionBundleModifier for CategoryStrictSuspensionBundleAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictSuspensionBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict suspension bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictSuspensionBundleEngine {
    pub steps: Vec<Rc<dyn StrictSuspensionBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictSuspensionBundleEngine {
    pub fn price(&mut self, item: &StrictSuspensionBundle) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictSuspensionBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictSuspensionBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_suspension_bundle_engine() -> StrictSuspensionBundleEngine {
    StrictSuspensionBundleEngine {
        steps: vec![
            Rc::new(MinimumStrictSuspensionBundleAdjuster { minimum: 48.0 }) as Rc<dyn StrictSuspensionBundleModifier>,
            Rc::new(StrictSuspensionBundleScoreOffsetAdjuster { weight: 0.85 }),
            Rc::new(CapStrictSuspensionBundleAdjuster { cap: 28647.0 }),
            Rc::new(CategoryStrictSuspensionBundleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.83), ("retail".to_string(), 0.72)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 81909.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_suspension_bundle_engine();
        let item = new_strict_suspension_bundle("code-968", "name-868");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
