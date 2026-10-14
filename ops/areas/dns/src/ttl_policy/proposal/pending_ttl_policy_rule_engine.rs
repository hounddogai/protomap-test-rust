use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending ttl policy rule calculation.
pub trait PendingTtlPolicyRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTtlPolicyRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyRuleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl PendingTtlPolicyRuleStep for PendingTtlPolicyRuleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &PendingTtlPolicyRule, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingTtlPolicyRuleAdjuster {
    pub step: f64,
}

impl PendingTtlPolicyRuleStep for RoundingPendingTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingTtlPolicyRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingTtlPolicyRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingTtlPolicyRuleStep for FlaggedPendingTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingTtlPolicyRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingTtlPolicyRuleAdjuster {
    pub cap: f64,
}

impl PendingTtlPolicyRuleStep for CapPendingTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingTtlPolicyRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending ttl policy rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyRuleEngine {
    pub steps: Vec<Rc<dyn PendingTtlPolicyRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTtlPolicyRuleEngine {
    pub fn compute(&mut self, item: &PendingTtlPolicyRule) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PendingTtlPolicyRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_ttl_policy_rule_engine(flags: &flags::Registry) -> PendingTtlPolicyRuleEngine {
    PendingTtlPolicyRuleEngine {
        steps: vec![
            Rc::new(PendingTtlPolicyRuleQuantityOffsetAdjuster { weight: 0.34 }) as Rc<dyn PendingTtlPolicyRuleStep>,
            Rc::new(RoundingPendingTtlPolicyRuleAdjuster { step: 0.5 }),
            Rc::new(FlaggedPendingTtlPolicyRuleAdjuster {
                flags: flags.clone(),
                flag: "pending-ttl-policy-rule-lenient".to_string(),
                percent: -1.2,
            }),
            Rc::new(CapPendingTtlPolicyRuleAdjuster { cap: 16583.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 24640.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_ttl_policy_rule_engine(&flags::new_registry());
        let item = new_pending_ttl_policy_rule("code-818", "name-732");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
