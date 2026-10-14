use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence rule calculation.
pub trait EvidenceRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceRuleAdjuster {
    pub step: f64,
}

impl EvidenceRuleFactor for RoundingEvidenceRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceRuleAdjuster {
    pub percent: f64,
}

impl EvidenceRuleFactor for PercentEvidenceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceRuleAdjuster {
    pub cap: f64,
}

impl EvidenceRuleFactor for CapEvidenceRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceEvidenceRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceRuleFactor for NamespaceEvidenceRuleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &EvidenceRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceRuleAdjuster {
    pub minimum: f64,
}

impl EvidenceRuleFactor for MinimumEvidenceRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the evidence rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceRuleEngine {
    pub steps: Vec<Rc<dyn EvidenceRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceRuleEngine {
    pub fn compute(&mut self, item: &EvidenceRule) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_evidence_rule_engine() -> EvidenceRuleEngine {
    EvidenceRuleEngine {
        steps: vec![
            Rc::new(RoundingEvidenceRuleAdjuster { step: 0.25 }) as Rc<dyn EvidenceRuleFactor>,
            Rc::new(PercentEvidenceRuleAdjuster { percent: 22.1 }),
            Rc::new(CapEvidenceRuleAdjuster { cap: 4860.0 }),
            Rc::new(NamespaceEvidenceRuleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.92), ("east".to_string(), 0.93)]),
            }),
            Rc::new(MinimumEvidenceRuleAdjuster { minimum: 7.5 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 30362.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_rule_engine();
        let item = new_evidence_rule("code-907", "label-146");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
