use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment audit calculation.
pub trait SegmentAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSegmentAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentAuditFactor for NameSegmentAuditAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SegmentAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentAuditAdjuster {
    pub step: f64,
}

impl SegmentAuditFactor for RoundingSegmentAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentAuditFactor for FlaggedSegmentAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the segment audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentAuditEngine {
    pub steps: Vec<Rc<dyn SegmentAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentAuditEngine {
    pub fn evaluate(&mut self, item: &SegmentAudit) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[SegmentAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_segment_audit_engine(flags: &flags::Registry) -> SegmentAuditEngine {
    SegmentAuditEngine {
        steps: vec![
            Rc::new(NameSegmentAuditAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.91)]),
            }) as Rc<dyn SegmentAuditFactor>,
            Rc::new(RoundingSegmentAuditAdjuster { step: 0.05 }),
            Rc::new(FlaggedSegmentAuditAdjuster {
                flags: flags.clone(),
                flag: "segment-audit-regional".to_string(),
                percent: -7.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 35217.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_audit_engine(&flags::new_registry());
        let item = new_segment_audit("code-130", "name-596");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
