use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding target calculation.
pub trait BindingTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBindingTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingTargetAdjuster for LocaleBindingTargetAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BindingTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingTargetAdjuster {
    pub percent: f64,
}

impl BindingTargetAdjuster for PercentBindingTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingTargetAdjuster {
    pub minimum: f64,
}

impl BindingTargetAdjuster for MinimumBindingTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingTargetAdjuster {
    pub step: f64,
}

impl BindingTargetAdjuster for RoundingBindingTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the binding target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingTargetEngine {
    pub steps: Vec<Rc<dyn BindingTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingTargetEngine {
    pub fn evaluate(&mut self, item: &BindingTarget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BindingTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_target_engine() -> BindingTargetEngine {
    BindingTargetEngine {
        steps: vec![
            Rc::new(LocaleBindingTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.73), ("south".to_string(), 1.39)]),
            }) as Rc<dyn BindingTargetAdjuster>,
            Rc::new(PercentBindingTargetAdjuster { percent: -6.7 }),
            Rc::new(MinimumBindingTargetAdjuster { minimum: 9.0 }),
            Rc::new(RoundingBindingTargetAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 82912.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_target_engine();
        let item = new_binding_target("code-318", "title-439");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
