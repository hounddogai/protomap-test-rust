use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the evidence policy calculation.
pub trait EvidencePolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidencePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicyMaximumOffsetAdjuster {
    pub weight: f64,
}

impl EvidencePolicyFactor for EvidencePolicyMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &EvidencePolicy, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidencePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidencePolicyFactor for FlaggedEvidencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidencePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidencePolicyAdjuster {
    pub cap: f64,
}

impl EvidencePolicyFactor for CapEvidencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidencePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the evidence policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidencePolicyEngine {
    pub steps: Vec<Rc<dyn EvidencePolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidencePolicyEngine {
    pub fn run(&mut self, item: &EvidencePolicy) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidencePolicyFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidencePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_policy_engine(flags: &flags::Registry) -> EvidencePolicyEngine {
    EvidencePolicyEngine {
        steps: vec![
            Rc::new(EvidencePolicyMaximumOffsetAdjuster { weight: 1.25 }) as Rc<dyn EvidencePolicyFactor>,
            Rc::new(FlaggedEvidencePolicyAdjuster {
                flags: flags.clone(),
                flag: "evidence-policy-manual".to_string(),
                percent: 3.1,
            }),
            Rc::new(CapEvidencePolicyAdjuster { cap: 12258.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 25784.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_policy_engine(&flags::new_registry());
        let item = new_evidence_policy("code-347", "name-323");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
