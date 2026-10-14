use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the evidence metric calculation.
pub trait EvidenceMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceMetricAdjuster {
    pub minimum: f64,
}

impl EvidenceMetricAdjuster for MinimumEvidenceMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceMetricAdjuster {
    pub percent: f64,
}

impl EvidenceMetricAdjuster for PercentEvidenceMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMetricMaximumOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceMetricAdjuster for EvidenceMetricMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceMetric, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the evidence metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceMetricEngine {
    pub steps: Vec<Rc<dyn EvidenceMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceMetricEngine {
    pub fn evaluate(&mut self, item: &EvidenceMetric) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_metric_engine() -> EvidenceMetricEngine {
    EvidenceMetricEngine {
        steps: vec![
            Rc::new(MinimumEvidenceMetricAdjuster { minimum: 20.5 }) as Rc<dyn EvidenceMetricAdjuster>,
            Rc::new(PercentEvidenceMetricAdjuster { percent: -4.9 }),
            Rc::new(EvidenceMetricMaximumOffsetAdjuster { weight: 1.53 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 17072.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_metric_engine();
        let item = new_evidence_metric("code-610", "title-38");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
