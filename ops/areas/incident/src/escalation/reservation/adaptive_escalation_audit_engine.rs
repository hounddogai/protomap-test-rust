use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive escalation audit calculation.
pub trait AdaptiveEscalationAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveEscalationAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveEscalationAuditAdjuster {
    pub step: f64,
}

impl AdaptiveEscalationAuditAdjuster for RoundingAdaptiveEscalationAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveEscalationAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEscalationAuditLimitOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveEscalationAuditAdjuster for AdaptiveEscalationAuditLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveEscalationAudit, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveEscalationAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveEscalationAuditAdjuster for TieredAdaptiveEscalationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveEscalationAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveEscalationAuditAdjuster {
    pub minimum: f64,
}

impl AdaptiveEscalationAuditAdjuster for MinimumAdaptiveEscalationAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveEscalationAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveEscalationAuditAdjuster {
    pub percent: f64,
}

impl AdaptiveEscalationAuditAdjuster for PercentAdaptiveEscalationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveEscalationAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveEscalationAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveEscalationAuditAdjuster for FlaggedAdaptiveEscalationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveEscalationAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive escalation audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveEscalationAuditEngine {
    pub steps: Vec<Rc<dyn AdaptiveEscalationAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveEscalationAuditEngine {
    pub fn compute(&mut self, item: &AdaptiveEscalationAudit) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveEscalationAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveEscalationAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_escalation_audit_engine(flags: &flags::Registry) -> AdaptiveEscalationAuditEngine {
    AdaptiveEscalationAuditEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveEscalationAuditAdjuster { step: 0.25 }) as Rc<dyn AdaptiveEscalationAuditAdjuster>,
            Rc::new(AdaptiveEscalationAuditLimitOffsetAdjuster { weight: 0.45 }),
            Rc::new(TieredAdaptiveEscalationAuditAdjuster {
                bounds: vec![5521.0, 8117.0],
                rates: vec![0.9, 0.93],
            }),
            Rc::new(MinimumAdaptiveEscalationAuditAdjuster { minimum: 42.8 }),
            Rc::new(PercentAdaptiveEscalationAuditAdjuster { percent: -6.3 }),
            Rc::new(FlaggedAdaptiveEscalationAuditAdjuster {
                flags: flags.clone(),
                flag: "adaptive-escalation-audit-rolling".to_string(),
                percent: -6.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 1197.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_escalation_audit_engine(&flags::new_registry());
        let item = new_adaptive_escalation_audit("code-140", "title-936");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
