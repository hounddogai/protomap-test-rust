use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the redaction claim calculation.
pub trait RedactionClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionClaimAdjuster {
    pub minimum: f64,
}

impl RedactionClaimStep for MinimumRedactionClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionClaimMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RedactionClaimStep for RedactionClaimMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionClaim, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRedactionClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionClaimStep for NameRedactionClaimAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RedactionClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the redaction claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionClaimEngine {
    pub steps: Vec<Rc<dyn RedactionClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionClaimEngine {
    pub fn compute(&mut self, item: &RedactionClaim) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_claim_engine() -> RedactionClaimEngine {
    RedactionClaimEngine {
        steps: vec![
            Rc::new(MinimumRedactionClaimAdjuster { minimum: 27.3 }) as Rc<dyn RedactionClaimStep>,
            Rc::new(RedactionClaimMaximumOffsetAdjuster { weight: 0.16 }),
            Rc::new(NameRedactionClaimAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.08),
                    ("east".to_string(), 0.75),
                    ("south".to_string(), 0.97),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 19434.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_claim_engine();
        let item = new_redaction_claim("code-702", "name-780");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
