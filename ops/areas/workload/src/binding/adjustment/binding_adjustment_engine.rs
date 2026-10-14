use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the binding adjustment calculation.
pub trait BindingAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBindingAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingAdjustmentFactor for NameBindingAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BindingAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAdjustmentDiscountOffsetAdjuster {
    pub weight: f64,
}

impl BindingAdjustmentFactor for BindingAdjustmentDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &BindingAdjustment, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingAdjustmentAdjuster {
    pub cap: f64,
}

impl BindingAdjustmentFactor for CapBindingAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingAdjustmentAdjuster {
    pub minimum: f64,
}

impl BindingAdjustmentFactor for MinimumBindingAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the binding adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingAdjustmentEngine {
    pub steps: Vec<Rc<dyn BindingAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingAdjustmentEngine {
    pub fn run(&mut self, item: &BindingAdjustment) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingAdjustmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_binding_adjustment_engine() -> BindingAdjustmentEngine {
    BindingAdjustmentEngine {
        steps: vec![
            Rc::new(NameBindingAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.9),
                    ("east".to_string(), 0.88),
                    ("retail".to_string(), 0.88),
                ]),
            }) as Rc<dyn BindingAdjustmentFactor>,
            Rc::new(BindingAdjustmentDiscountOffsetAdjuster { weight: 0.54 }),
            Rc::new(CapBindingAdjustmentAdjuster { cap: 14361.0 }),
            Rc::new(MinimumBindingAdjustmentAdjuster { minimum: 17.9 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 25624.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_adjustment_engine();
        let item = new_binding_adjustment("code-466", "name-853");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
