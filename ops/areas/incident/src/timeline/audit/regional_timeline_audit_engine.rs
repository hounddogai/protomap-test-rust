use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional timeline audit calculation.
pub trait RegionalTimelineAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalTimelineAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalTimelineAuditAdjuster {
    pub percent: f64,
}

impl RegionalTimelineAuditAdjuster for PercentRegionalTimelineAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalTimelineAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalTimelineAuditAdjuster {
    pub cap: f64,
}

impl RegionalTimelineAuditAdjuster for CapRegionalTimelineAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalTimelineAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineAuditDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl RegionalTimelineAuditAdjuster for RegionalTimelineAuditDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalTimelineAudit, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalTimelineAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalTimelineAuditAdjuster for FlaggedRegionalTimelineAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalTimelineAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional timeline audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalTimelineAuditEngine {
    pub steps: Vec<Rc<dyn RegionalTimelineAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalTimelineAuditEngine {
    pub fn compute(&mut self, item: &RegionalTimelineAudit) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionalTimelineAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_timeline_audit_engine(flags: &flags::Registry) -> RegionalTimelineAuditEngine {
    RegionalTimelineAuditEngine {
        steps: vec![
            Rc::new(PercentRegionalTimelineAuditAdjuster { percent: -5.4 }) as Rc<dyn RegionalTimelineAuditAdjuster>,
            Rc::new(CapRegionalTimelineAuditAdjuster { cap: 48963.0 }),
            Rc::new(RegionalTimelineAuditDurationMinutesOffsetAdjuster { weight: 0.01 }),
            Rc::new(FlaggedRegionalTimelineAuditAdjuster {
                flags: flags.clone(),
                flag: "regional-timeline-audit-batched".to_string(),
                percent: 6.3,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 51864.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_timeline_audit_engine(&flags::new_registry());
        let item = new_regional_timeline_audit("code-816", "title-470");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
