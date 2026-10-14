use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event baseline calculation.
pub trait AuditEventBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventBaselineAdjuster {
    pub step: f64,
}

impl AuditEventBaselineModifier for RoundingAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventBaselineAdjuster {
    pub minimum: f64,
}

impl AuditEventBaselineModifier for MinimumAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventBaselineModifier for FlaggedAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAuditEventBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AuditEventBaselineModifier for TitleAuditEventBaselineAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AuditEventBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the audit event baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventBaselineEngine {
    pub steps: Vec<Rc<dyn AuditEventBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventBaselineEngine {
    pub fn run(&mut self, item: &AuditEventBaseline) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_audit_event_baseline_engine(flags: &flags::Registry) -> AuditEventBaselineEngine {
    AuditEventBaselineEngine {
        steps: vec![
            Rc::new(RoundingAuditEventBaselineAdjuster { step: 0.5 }) as Rc<dyn AuditEventBaselineModifier>,
            Rc::new(MinimumAuditEventBaselineAdjuster { minimum: 31.0 }),
            Rc::new(FlaggedAuditEventBaselineAdjuster {
                flags: flags.clone(),
                flag: "audit-event-baseline-seasonal".to_string(),
                percent: -1.3,
            }),
            Rc::new(TitleAuditEventBaselineAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 75628.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_baseline_engine(&flags::new_registry());
        let item = new_audit_event_baseline("code-231", "title-257");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
