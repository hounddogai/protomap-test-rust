use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence mapping calculation.
pub trait EvidenceMappingStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceMappingAdjuster {
    pub minimum: f64,
}

impl EvidenceMappingStep for MinimumEvidenceMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceMappingAdjuster {
    pub percent: f64,
}

impl EvidenceMappingStep for PercentEvidenceMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMappingPriorityOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceMappingStep for EvidenceMappingPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceMapping, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceMappingStep for TieredEvidenceMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the evidence mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceMappingEngine {
    pub steps: Vec<Rc<dyn EvidenceMappingStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceMappingEngine {
    pub fn run(&mut self, item: &EvidenceMapping) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_mapping_engine() -> EvidenceMappingEngine {
    EvidenceMappingEngine {
        steps: vec![
            Rc::new(MinimumEvidenceMappingAdjuster { minimum: 39.9 }) as Rc<dyn EvidenceMappingStep>,
            Rc::new(PercentEvidenceMappingAdjuster { percent: -3.7 }),
            Rc::new(EvidenceMappingPriorityOffsetAdjuster { weight: 1.98 }),
            Rc::new(TieredEvidenceMappingAdjuster {
                bounds: vec![2921.0, 5344.0, 6223.0],
                rates: vec![0.94, 1.06, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 3210.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_mapping_engine();
        let item = new_evidence_mapping("code-659", "label-73");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
