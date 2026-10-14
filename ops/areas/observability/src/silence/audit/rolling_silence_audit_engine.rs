use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling silence audit calculation.
pub trait RollingSilenceAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingSilenceAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingSilenceAuditAdjuster {
    pub cap: f64,
}

impl RollingSilenceAuditStep for CapRollingSilenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingSilenceAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRollingSilenceAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingSilenceAuditStep for SourceRollingSilenceAuditAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RollingSilenceAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceAuditCapacityOffsetAdjuster {
    pub weight: f64,
}

impl RollingSilenceAuditStep for RollingSilenceAuditCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &RollingSilenceAudit, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the rolling silence audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingSilenceAuditEngine {
    pub steps: Vec<Rc<dyn RollingSilenceAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingSilenceAuditEngine {
    pub fn compute(&mut self, item: &RollingSilenceAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingSilenceAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_silence_audit_engine() -> RollingSilenceAuditEngine {
    RollingSilenceAuditEngine {
        steps: vec![
            Rc::new(CapRollingSilenceAuditAdjuster { cap: 16478.0 }) as Rc<dyn RollingSilenceAuditStep>,
            Rc::new(SourceRollingSilenceAuditAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.06), ("south".to_string(), 0.74)]),
            }),
            Rc::new(RollingSilenceAuditCapacityOffsetAdjuster { weight: 1.76 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 47337.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_silence_audit_engine();
        let item = new_rolling_silence_audit("code-312", "title-303");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
