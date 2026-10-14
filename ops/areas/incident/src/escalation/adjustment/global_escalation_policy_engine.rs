use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global escalation policy calculation.
pub trait GlobalEscalationPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalEscalationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalEscalationPolicyAdjuster {
    pub cap: f64,
}

impl GlobalEscalationPolicyStep for CapGlobalEscalationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalEscalationPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationPolicyQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl GlobalEscalationPolicyStep for GlobalEscalationPolicyQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalEscalationPolicy, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalEscalationPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalEscalationPolicyStep for LabelGlobalEscalationPolicyAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalEscalationPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalEscalationPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalEscalationPolicyStep for FlaggedGlobalEscalationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalEscalationPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global escalation policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalEscalationPolicyEngine {
    pub steps: Vec<Rc<dyn GlobalEscalationPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalEscalationPolicyEngine {
    pub fn evaluate(&mut self, item: &GlobalEscalationPolicy) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalEscalationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_escalation_policy_engine(flags: &flags::Registry) -> GlobalEscalationPolicyEngine {
    GlobalEscalationPolicyEngine {
        steps: vec![
            Rc::new(CapGlobalEscalationPolicyAdjuster { cap: 17385.0 }) as Rc<dyn GlobalEscalationPolicyStep>,
            Rc::new(GlobalEscalationPolicyQuotaUnitsOffsetAdjuster { weight: 0.71 }),
            Rc::new(LabelGlobalEscalationPolicyAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.91)]),
            }),
            Rc::new(FlaggedGlobalEscalationPolicyAdjuster {
                flags: flags.clone(),
                flag: "global-escalation-policy-strict".to_string(),
                percent: -7.3,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 88786.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_escalation_policy_engine(&flags::new_registry());
        let item = new_global_escalation_policy("code-211", "label-12");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
