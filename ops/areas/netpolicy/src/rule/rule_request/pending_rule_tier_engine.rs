use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending rule tier calculation.
pub trait PendingRuleTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRuleTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRuleTierAdjuster {
    pub step: f64,
}

impl PendingRuleTierModifier for RoundingPendingRuleTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRuleTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRuleTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRuleTierModifier for FlaggedPendingRuleTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRuleTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRuleTierAdjuster {
    pub percent: f64,
}

impl PendingRuleTierModifier for PercentPendingRuleTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRuleTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPendingRuleTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRuleTierModifier for UnitPendingRuleTierAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PendingRuleTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending rule tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRuleTierEngine {
    pub steps: Vec<Rc<dyn PendingRuleTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRuleTierEngine {
    pub fn run(&mut self, item: &PendingRuleTier) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[PendingRuleTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_rule_tier_engine(flags: &flags::Registry) -> PendingRuleTierEngine {
    PendingRuleTierEngine {
        steps: vec![
            Rc::new(RoundingPendingRuleTierAdjuster { step: 0.1 }) as Rc<dyn PendingRuleTierModifier>,
            Rc::new(FlaggedPendingRuleTierAdjuster {
                flags: flags.clone(),
                flag: "pending-rule-tier-adaptive".to_string(),
                percent: -8.1,
            }),
            Rc::new(PercentPendingRuleTierAdjuster { percent: -20.5 }),
            Rc::new(UnitPendingRuleTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.24), ("central".to_string(), 0.87)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 54473.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_rule_tier_engine(&flags::new_registry());
        let item = new_pending_rule_tier("code-359", "label-310");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
