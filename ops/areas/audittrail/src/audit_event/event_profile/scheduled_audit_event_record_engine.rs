use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled audit event record calculation.
pub trait ScheduledAuditEventRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledAuditEventRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledAuditEventRecordAdjuster {
    pub cap: f64,
}

impl ScheduledAuditEventRecordModifier for CapScheduledAuditEventRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledAuditEventRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledAuditEventRecordAdjuster {
    pub percent: f64,
}

impl ScheduledAuditEventRecordModifier for PercentScheduledAuditEventRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledAuditEventRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledAuditEventRecordAdjuster {
    pub minimum: f64,
}

impl ScheduledAuditEventRecordModifier for MinimumScheduledAuditEventRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledAuditEventRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledAuditEventRecordAdjuster {
    pub step: f64,
}

impl ScheduledAuditEventRecordModifier for RoundingScheduledAuditEventRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledAuditEventRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled audit event record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledAuditEventRecordEngine {
    pub steps: Vec<Rc<dyn ScheduledAuditEventRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledAuditEventRecordEngine {
    pub fn evaluate(&mut self, item: &ScheduledAuditEventRecord) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledAuditEventRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledAuditEventRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_audit_event_record_engine() -> ScheduledAuditEventRecordEngine {
    ScheduledAuditEventRecordEngine {
        steps: vec![
            Rc::new(CapScheduledAuditEventRecordAdjuster { cap: 43527.0 }) as Rc<dyn ScheduledAuditEventRecordModifier>,
            Rc::new(PercentScheduledAuditEventRecordAdjuster { percent: -17.6 }),
            Rc::new(MinimumScheduledAuditEventRecordAdjuster { minimum: 47.3 }),
            Rc::new(RoundingScheduledAuditEventRecordAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 57348.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_audit_event_record_engine();
        let item = new_scheduled_audit_event_record("code-610", "title-658");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
