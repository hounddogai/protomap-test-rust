use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default audit event entry calculation.
pub trait DefaultAuditEventEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultAuditEventEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultAuditEventEntryAdjuster {
    pub step: f64,
}

impl DefaultAuditEventEntryAdjuster for RoundingDefaultAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultAuditEventEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultAuditEventEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultAuditEventEntryAdjuster for FlaggedDefaultAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultAuditEventEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultAuditEventEntryAdjuster {
    pub minimum: f64,
}

impl DefaultAuditEventEntryAdjuster for MinimumDefaultAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultAuditEventEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultAuditEventEntryAdjuster {
    pub percent: f64,
}

impl DefaultAuditEventEntryAdjuster for PercentDefaultAuditEventEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultAuditEventEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default audit event entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultAuditEventEntryEngine {
    pub steps: Vec<Rc<dyn DefaultAuditEventEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultAuditEventEntryEngine {
    pub fn evaluate(&mut self, item: &DefaultAuditEventEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultAuditEventEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultAuditEventEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_audit_event_entry_engine(flags: &flags::Registry) -> DefaultAuditEventEntryEngine {
    DefaultAuditEventEntryEngine {
        steps: vec![
            Rc::new(RoundingDefaultAuditEventEntryAdjuster { step: 0.05 }) as Rc<dyn DefaultAuditEventEntryAdjuster>,
            Rc::new(FlaggedDefaultAuditEventEntryAdjuster {
                flags: flags.clone(),
                flag: "default-audit-event-entry-global".to_string(),
                percent: 5.2,
            }),
            Rc::new(MinimumDefaultAuditEventEntryAdjuster { minimum: 27.4 }),
            Rc::new(PercentDefaultAuditEventEntryAdjuster { percent: 10.3 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 11231.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_audit_event_entry_engine(&flags::new_registry());
        let item = new_default_audit_event_entry("code-377", "label-176");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
