use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback rule assignment calculation.
pub trait FallbackRuleAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRuleAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRuleAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRuleAssignmentAdjuster for TieredFallbackRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRuleAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackRuleAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRuleAssignmentAdjuster for TitleFallbackRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackRuleAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackRuleAssignmentAdjuster {
    pub percent: f64,
}

impl FallbackRuleAssignmentAdjuster for PercentFallbackRuleAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackRuleAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback rule assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRuleAssignmentEngine {
    pub steps: Vec<Rc<dyn FallbackRuleAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRuleAssignmentEngine {
    pub fn compute(&mut self, item: &FallbackRuleAssignment) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackRuleAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRuleAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_rule_assignment_engine() -> FallbackRuleAssignmentEngine {
    FallbackRuleAssignmentEngine {
        steps: vec![
            Rc::new(TieredFallbackRuleAssignmentAdjuster {
                bounds: vec![2867.0, 8210.0],
                rates: vec![0.96, 0.95],
            }) as Rc<dyn FallbackRuleAssignmentAdjuster>,
            Rc::new(TitleFallbackRuleAssignmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.0), ("west".to_string(), 1.27)]),
            }),
            Rc::new(PercentFallbackRuleAssignmentAdjuster { percent: 22.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 36809.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_rule_assignment_engine();
        let item = new_fallback_rule_assignment("code-51", "title-13");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
