use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched policy template calculation.
pub trait BatchedPolicyTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPolicyTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedPolicyTemplateAdjuster {
    pub percent: f64,
}

impl BatchedPolicyTemplateModifier for PercentBatchedPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedPolicyTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedPolicyTemplateAdjuster {
    pub step: f64,
}

impl BatchedPolicyTemplateModifier for RoundingBatchedPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedPolicyTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPolicyTemplateAdjuster {
    pub minimum: f64,
}

impl BatchedPolicyTemplateModifier for MinimumBatchedPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPolicyTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedPolicyTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedPolicyTemplateModifier for LabelBatchedPolicyTemplateAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedPolicyTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched policy template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPolicyTemplateEngine {
    pub steps: Vec<Rc<dyn BatchedPolicyTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPolicyTemplateEngine {
    pub fn run(&mut self, item: &BatchedPolicyTemplate) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedPolicyTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedPolicyTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_policy_template_engine() -> BatchedPolicyTemplateEngine {
    BatchedPolicyTemplateEngine {
        steps: vec![
            Rc::new(PercentBatchedPolicyTemplateAdjuster { percent: 19.9 }) as Rc<dyn BatchedPolicyTemplateModifier>,
            Rc::new(RoundingBatchedPolicyTemplateAdjuster { step: 0.5 }),
            Rc::new(MinimumBatchedPolicyTemplateAdjuster { minimum: 38.8 }),
            Rc::new(LabelBatchedPolicyTemplateAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.07),
                    ("central".to_string(), 1.06),
                    ("online".to_string(), 1.24),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 25987.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_policy_template_engine();
        let item = new_batched_policy_template("code-80", "label-452");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
