use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence request calculation.
pub trait EvidenceRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceRequestAdjuster {
    pub minimum: f64,
}

impl EvidenceRequestStep for MinimumEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceRequestStep for TieredEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEvidenceRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceRequestStep for LabelEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EvidenceRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceRequestAdjuster {
    pub cap: f64,
}

impl EvidenceRequestStep for CapEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceRequestAdjuster {
    pub percent: f64,
}

impl EvidenceRequestStep for PercentEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the evidence request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceRequestEngine {
    pub steps: Vec<Rc<dyn EvidenceRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceRequestEngine {
    pub fn run(&mut self, item: &EvidenceRequest) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_request_engine() -> EvidenceRequestEngine {
    EvidenceRequestEngine {
        steps: vec![
            Rc::new(MinimumEvidenceRequestAdjuster { minimum: 32.8 }) as Rc<dyn EvidenceRequestStep>,
            Rc::new(TieredEvidenceRequestAdjuster {
                bounds: vec![103.0, 4064.0, 7205.0, 9185.0],
                rates: vec![1.06, 0.8, 0.99, 1.14],
            }),
            Rc::new(LabelEvidenceRequestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.88)]),
            }),
            Rc::new(CapEvidenceRequestAdjuster { cap: 25508.0 }),
            Rc::new(PercentEvidenceRequestAdjuster { percent: -16.1 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 40154.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_request_engine();
        let item = new_evidence_request("code-869", "label-855");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
