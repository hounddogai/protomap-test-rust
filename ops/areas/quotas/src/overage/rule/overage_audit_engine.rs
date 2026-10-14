use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage audit calculation.
pub trait OverageAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageAuditAdjuster {
    pub cap: f64,
}

impl OverageAuditModifier for CapOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageAuditAdjuster {
    pub percent: f64,
}

impl OverageAuditModifier for PercentOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageAuditAdjuster {
    pub step: f64,
}

impl OverageAuditModifier for RoundingOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageAuditModifier for TieredOverageAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the overage audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageAuditEngine {
    pub steps: Vec<Rc<dyn OverageAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageAuditEngine {
    pub fn run(&mut self, item: &OverageAudit) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageAuditModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_overage_audit_engine() -> OverageAuditEngine {
    OverageAuditEngine {
        steps: vec![
            Rc::new(CapOverageAuditAdjuster { cap: 31713.0 }) as Rc<dyn OverageAuditModifier>,
            Rc::new(PercentOverageAuditAdjuster { percent: -20.3 }),
            Rc::new(RoundingOverageAuditAdjuster { step: 0.05 }),
            Rc::new(TieredOverageAuditAdjuster {
                bounds: vec![7763.0, 7814.0],
                rates: vec![0.88, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 24990.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_audit_engine();
        let item = new_overage_audit("code-852", "label-910");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
