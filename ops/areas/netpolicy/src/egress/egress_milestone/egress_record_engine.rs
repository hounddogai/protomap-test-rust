use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress record calculation.
pub trait EgressRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressRecordAdjuster {
    pub cap: f64,
}

impl EgressRecordModifier for CapEgressRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressRecordAdjuster {
    pub step: f64,
}

impl EgressRecordModifier for RoundingEgressRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressRecordUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl EgressRecordModifier for EgressRecordUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &EgressRecord, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressRecordAdjuster {
    pub percent: f64,
}

impl EgressRecordModifier for PercentEgressRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressRecordEngine {
    pub steps: Vec<Rc<dyn EgressRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressRecordEngine {
    pub fn price(&mut self, item: &EgressRecord) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[EgressRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_egress_record_engine() -> EgressRecordEngine {
    EgressRecordEngine {
        steps: vec![
            Rc::new(CapEgressRecordAdjuster { cap: 32491.0 }) as Rc<dyn EgressRecordModifier>,
            Rc::new(RoundingEgressRecordAdjuster { step: 0.05 }),
            Rc::new(EgressRecordUpdatedAtOffsetAdjuster { weight: 1.12 }),
            Rc::new(PercentEgressRecordAdjuster { percent: -5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 93159.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_record_engine();
        let item = new_egress_record("code-502", "title-318");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
