use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event request calculation.
pub trait AuditEventRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventRequestAdjuster {
    pub step: f64,
}

impl AuditEventRequestAdjuster for RoundingAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventRequestAdjuster {
    pub minimum: f64,
}

impl AuditEventRequestAdjuster for MinimumAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventRequestAdjuster for TieredAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventRequestAdjuster for FlaggedAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventRequestAdjuster {
    pub cap: f64,
}

impl AuditEventRequestAdjuster for CapAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAuditEventRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventRequestAdjuster for LabelAuditEventRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AuditEventRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the audit event request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventRequestEngine {
    pub steps: Vec<Rc<dyn AuditEventRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventRequestEngine {
    pub fn compute(&mut self, item: &AuditEventRequest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventRequestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_request_engine(flags: &flags::Registry) -> AuditEventRequestEngine {
    AuditEventRequestEngine {
        steps: vec![
            Rc::new(RoundingAuditEventRequestAdjuster { step: 0.25 }) as Rc<dyn AuditEventRequestAdjuster>,
            Rc::new(MinimumAuditEventRequestAdjuster { minimum: 2.3 }),
            Rc::new(TieredAuditEventRequestAdjuster {
                bounds: vec![1419.0, 5958.0, 6281.0, 6337.0],
                rates: vec![0.92, 1.18, 0.91, 1.01],
            }),
            Rc::new(FlaggedAuditEventRequestAdjuster {
                flags: flags.clone(),
                flag: "audit-event-request-global".to_string(),
                percent: -5.8,
            }),
            Rc::new(CapAuditEventRequestAdjuster { cap: 23001.0 }),
            Rc::new(LabelAuditEventRequestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.92), ("west".to_string(), 1.07)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 57767.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_request_engine(&flags::new_registry());
        let item = new_audit_event_request("code-671", "label-792");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
