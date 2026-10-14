use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global audit event notice calculation.
pub trait GlobalAuditEventNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAuditEventNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalAuditEventNoticeAdjuster {
    pub cap: f64,
}

impl GlobalAuditEventNoticeFactor for CapGlobalAuditEventNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalAuditEventNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalAuditEventNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalAuditEventNoticeFactor for TieredGlobalAuditEventNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalAuditEventNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalAuditEventNoticeAdjuster {
    pub percent: f64,
}

impl GlobalAuditEventNoticeFactor for PercentGlobalAuditEventNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalAuditEventNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalAuditEventNoticeAdjuster {
    pub step: f64,
}

impl GlobalAuditEventNoticeFactor for RoundingGlobalAuditEventNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalAuditEventNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global audit event notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAuditEventNoticeEngine {
    pub steps: Vec<Rc<dyn GlobalAuditEventNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAuditEventNoticeEngine {
    pub fn compute(&mut self, item: &GlobalAuditEventNotice) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalAuditEventNoticeFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAuditEventNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_audit_event_notice_engine() -> GlobalAuditEventNoticeEngine {
    GlobalAuditEventNoticeEngine {
        steps: vec![
            Rc::new(CapGlobalAuditEventNoticeAdjuster { cap: 5549.0 }) as Rc<dyn GlobalAuditEventNoticeFactor>,
            Rc::new(TieredGlobalAuditEventNoticeAdjuster {
                bounds: vec![788.0, 5569.0, 7030.0],
                rates: vec![0.85, 0.82, 0.91],
            }),
            Rc::new(PercentGlobalAuditEventNoticeAdjuster { percent: -4.4 }),
            Rc::new(RoundingGlobalAuditEventNoticeAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 95903.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_audit_event_notice_engine();
        let item = new_global_audit_event_notice("code-948", "title-819");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
