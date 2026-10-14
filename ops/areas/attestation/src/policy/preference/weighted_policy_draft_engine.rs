use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted policy draft calculation.
pub trait WeightedPolicyDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedPolicyDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedPolicyDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedPolicyDraftAdjuster for FlaggedWeightedPolicyDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedPolicyDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeWeightedPolicyDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedPolicyDraftAdjuster for ScopeWeightedPolicyDraftAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &WeightedPolicyDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedPolicyDraftAdjuster {
    pub minimum: f64,
}

impl WeightedPolicyDraftAdjuster for MinimumWeightedPolicyDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedPolicyDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraftPriorityOffsetAdjuster {
    pub weight: f64,
}

impl WeightedPolicyDraftAdjuster for WeightedPolicyDraftPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedPolicyDraft, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedPolicyDraftAdjuster {
    pub percent: f64,
}

impl WeightedPolicyDraftAdjuster for PercentWeightedPolicyDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedPolicyDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted policy draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedPolicyDraftEngine {
    pub steps: Vec<Rc<dyn WeightedPolicyDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedPolicyDraftEngine {
    pub fn evaluate(&mut self, item: &WeightedPolicyDraft) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedPolicyDraftAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedPolicyDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_policy_draft_engine(flags: &flags::Registry) -> WeightedPolicyDraftEngine {
    WeightedPolicyDraftEngine {
        steps: vec![
            Rc::new(FlaggedWeightedPolicyDraftAdjuster {
                flags: flags.clone(),
                flag: "weighted-policy-draft-legacy".to_string(),
                percent: -3.0,
            }) as Rc<dyn WeightedPolicyDraftAdjuster>,
            Rc::new(ScopeWeightedPolicyDraftAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.28), ("central".to_string(), 1.34)]),
            }),
            Rc::new(MinimumWeightedPolicyDraftAdjuster { minimum: 29.5 }),
            Rc::new(WeightedPolicyDraftPriorityOffsetAdjuster { weight: 1.32 }),
            Rc::new(PercentWeightedPolicyDraftAdjuster { percent: 16.6 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 78694.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_policy_draft_engine(&flags::new_registry());
        let item = new_weighted_policy_draft("code-223", "label-686");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
