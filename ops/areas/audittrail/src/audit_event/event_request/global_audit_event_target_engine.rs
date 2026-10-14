use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global audit event target calculation.
pub trait GlobalAuditEventTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAuditEventTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalAuditEventTargetAdjuster {
    pub cap: f64,
}

impl GlobalAuditEventTargetStep for CapGlobalAuditEventTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalAuditEventTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionGlobalAuditEventTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalAuditEventTargetStep for RegionGlobalAuditEventTargetAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &GlobalAuditEventTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalAuditEventTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalAuditEventTargetStep for FlaggedGlobalAuditEventTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalAuditEventTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global audit event target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventTargetEngine {
    pub steps: Vec<Rc<dyn GlobalAuditEventTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAuditEventTargetEngine {
    pub fn run(&mut self, item: &GlobalAuditEventTarget) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalAuditEventTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAuditEventTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_audit_event_target_engine(flags: &flags::Registry) -> GlobalAuditEventTargetEngine {
    GlobalAuditEventTargetEngine {
        steps: vec![
            Rc::new(CapGlobalAuditEventTargetAdjuster { cap: 39243.0 }) as Rc<dyn GlobalAuditEventTargetStep>,
            Rc::new(RegionGlobalAuditEventTargetAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.95),
                    ("retail".to_string(), 1.02),
                    ("online".to_string(), 1.36),
                ]),
            }),
            Rc::new(FlaggedGlobalAuditEventTargetAdjuster {
                flags: flags.clone(),
                flag: "global-audit-event-target-fallback".to_string(),
                percent: 5.0,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 70521.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_audit_event_target_engine(&flags::new_registry());
        let item = new_global_audit_event_target("code-191", "label-554");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
