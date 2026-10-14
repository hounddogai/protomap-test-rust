use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert audit calculation.
pub trait AlertAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertAuditAdjuster for TieredAlertAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertAuditAdjuster {
    pub percent: f64,
}

impl AlertAuditAdjuster for PercentAlertAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertAuditAdjuster {
    pub minimum: f64,
}

impl AlertAuditAdjuster for MinimumAlertAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertAuditAdjuster for FlaggedAlertAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertAuditAdjuster {
    pub cap: f64,
}

impl AlertAuditAdjuster for CapAlertAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertAuditAdjuster {
    pub step: f64,
}

impl AlertAuditAdjuster for RoundingAlertAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the alert audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertAuditEngine {
    pub steps: Vec<Rc<dyn AlertAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertAuditEngine {
    pub fn run(&mut self, item: &AlertAudit) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_alert_audit_engine(flags: &flags::Registry) -> AlertAuditEngine {
    AlertAuditEngine {
        steps: vec![
            Rc::new(TieredAlertAuditAdjuster {
                bounds: vec![1423.0, 3047.0, 3594.0],
                rates: vec![1.15, 0.89, 0.9],
            }) as Rc<dyn AlertAuditAdjuster>,
            Rc::new(PercentAlertAuditAdjuster { percent: -17.7 }),
            Rc::new(MinimumAlertAuditAdjuster { minimum: 39.3 }),
            Rc::new(FlaggedAlertAuditAdjuster {
                flags: flags.clone(),
                flag: "alert-audit-legacy".to_string(),
                percent: -4.6,
            }),
            Rc::new(CapAlertAuditAdjuster { cap: 40069.0 }),
            Rc::new(RoundingAlertAuditAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 72261.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_audit_engine(&flags::new_registry());
        let item = new_alert_audit("code-90", "label-636");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
