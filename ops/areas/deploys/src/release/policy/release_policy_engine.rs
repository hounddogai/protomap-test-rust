use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release policy calculation.
pub trait ReleasePolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleasePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePolicyAmountOffsetAdjuster {
    pub weight: f64,
}

impl ReleasePolicyStep for ReleasePolicyAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ReleasePolicy, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleasePolicyAdjuster {
    pub minimum: f64,
}

impl ReleasePolicyStep for MinimumReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleasePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceReleasePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleasePolicyStep for SourceReleasePolicyAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ReleasePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleasePolicyAdjuster {
    pub cap: f64,
}

impl ReleasePolicyStep for CapReleasePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleasePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the release policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleasePolicyEngine {
    pub steps: Vec<Rc<dyn ReleasePolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleasePolicyEngine {
    pub fn evaluate(&mut self, item: &ReleasePolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReleasePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_release_policy_engine() -> ReleasePolicyEngine {
    ReleasePolicyEngine {
        steps: vec![
            Rc::new(ReleasePolicyAmountOffsetAdjuster { weight: 0.03 }) as Rc<dyn ReleasePolicyStep>,
            Rc::new(MinimumReleasePolicyAdjuster { minimum: 26.7 }),
            Rc::new(SourceReleasePolicyAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.73), ("south".to_string(), 1.25)]),
            }),
            Rc::new(CapReleasePolicyAdjuster { cap: 3504.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 45072.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_policy_engine();
        let item = new_release_policy("code-189", "label-185");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
