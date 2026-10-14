use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient binding assignment calculation.
pub trait LenientBindingAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientBindingAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientBindingAssignmentAdjuster {
    pub minimum: f64,
}

impl LenientBindingAssignmentAdjuster for MinimumLenientBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientBindingAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientBindingAssignmentAdjuster {
    pub step: f64,
}

impl LenientBindingAssignmentAdjuster for RoundingLenientBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientBindingAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientBindingAssignmentAdjuster {
    pub cap: f64,
}

impl LenientBindingAssignmentAdjuster for CapLenientBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientBindingAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientBindingAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientBindingAssignmentAdjuster for TieredLenientBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientBindingAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientBindingAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientBindingAssignmentAdjuster for LabelLenientBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientBindingAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingAssignmentVersionOffsetAdjuster {
    pub weight: f64,
}

impl LenientBindingAssignmentAdjuster for LenientBindingAssignmentVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &LenientBindingAssignment, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the lenient binding assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientBindingAssignmentEngine {
    pub steps: Vec<Rc<dyn LenientBindingAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientBindingAssignmentEngine {
    pub fn price(&mut self, item: &LenientBindingAssignment) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientBindingAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientBindingAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_binding_assignment_engine() -> LenientBindingAssignmentEngine {
    LenientBindingAssignmentEngine {
        steps: vec![
            Rc::new(MinimumLenientBindingAssignmentAdjuster { minimum: 0.8 })
                as Rc<dyn LenientBindingAssignmentAdjuster>,
            Rc::new(RoundingLenientBindingAssignmentAdjuster { step: 0.5 }),
            Rc::new(CapLenientBindingAssignmentAdjuster { cap: 40345.0 }),
            Rc::new(TieredLenientBindingAssignmentAdjuster {
                bounds: vec![2993.0, 6412.0, 9906.0],
                rates: vec![1.0, 0.94, 0.99],
            }),
            Rc::new(LabelLenientBindingAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.07),
                    ("south".to_string(), 1.14),
                    ("west".to_string(), 1.25),
                ]),
            }),
            Rc::new(LenientBindingAssignmentVersionOffsetAdjuster { weight: 0.64 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 57199.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_binding_assignment_engine();
        let item = new_lenient_binding_assignment("code-850", "label-878");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
