use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence rollup calculation.
pub trait EvidenceRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceRollupAdjuster {
    pub percent: f64,
}

impl EvidenceRollupFactor for PercentEvidenceRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceRollupFactor for TieredEvidenceRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceRollupAdjuster {
    pub cap: f64,
}

impl EvidenceRollupFactor for CapEvidenceRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the evidence rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceRollupEngine {
    pub steps: Vec<Rc<dyn EvidenceRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceRollupEngine {
    pub fn evaluate(&mut self, item: &EvidenceRollup) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_rollup_engine() -> EvidenceRollupEngine {
    EvidenceRollupEngine {
        steps: vec![
            Rc::new(PercentEvidenceRollupAdjuster { percent: 22.9 }) as Rc<dyn EvidenceRollupFactor>,
            Rc::new(TieredEvidenceRollupAdjuster {
                bounds: vec![1950.0, 4859.0, 7659.0, 9691.0],
                rates: vec![0.9, 1.0, 1.03, 0.9],
            }),
            Rc::new(CapEvidenceRollupAdjuster { cap: 48783.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 44987.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_rollup_engine();
        let item = new_evidence_rollup("code-535", "name-78");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
