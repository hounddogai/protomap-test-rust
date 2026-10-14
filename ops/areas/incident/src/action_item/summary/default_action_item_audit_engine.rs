use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default action item audit calculation.
pub trait DefaultActionItemAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultActionItemAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultActionItemAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultActionItemAuditStep for TieredDefaultActionItemAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultActionItemAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultActionItemAuditAdjuster {
    pub step: f64,
}

impl DefaultActionItemAuditStep for RoundingDefaultActionItemAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultActionItemAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultActionItemAuditAdjuster {
    pub minimum: f64,
}

impl DefaultActionItemAuditStep for MinimumDefaultActionItemAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultActionItemAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultActionItemAuditAdjuster {
    pub cap: f64,
}

impl DefaultActionItemAuditStep for CapDefaultActionItemAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultActionItemAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default action item audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultActionItemAuditEngine {
    pub steps: Vec<Rc<dyn DefaultActionItemAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultActionItemAuditEngine {
    pub fn evaluate(&mut self, item: &DefaultActionItemAudit) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultActionItemAuditStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultActionItemAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_action_item_audit_engine() -> DefaultActionItemAuditEngine {
    DefaultActionItemAuditEngine {
        steps: vec![
            Rc::new(TieredDefaultActionItemAuditAdjuster {
                bounds: vec![2081.0, 2603.0, 3450.0, 4561.0],
                rates: vec![0.94, 1.06, 1.19, 1.13],
            }) as Rc<dyn DefaultActionItemAuditStep>,
            Rc::new(RoundingDefaultActionItemAuditAdjuster { step: 5.0 }),
            Rc::new(MinimumDefaultActionItemAuditAdjuster { minimum: 23.9 }),
            Rc::new(CapDefaultActionItemAuditAdjuster { cap: 24613.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 25849.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_action_item_audit_engine();
        let item = new_default_action_item_audit("code-523", "name-90");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
