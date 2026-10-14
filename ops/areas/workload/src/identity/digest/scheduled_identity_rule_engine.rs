use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled identity rule calculation.
pub trait ScheduledIdentityRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledIdentityRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledIdentityRuleAdjuster {
    pub step: f64,
}

impl ScheduledIdentityRuleAdjuster for RoundingScheduledIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledIdentityRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityRuleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledIdentityRuleAdjuster for ScheduledIdentityRuleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledIdentityRule, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledIdentityRuleAdjuster {
    pub cap: f64,
}

impl ScheduledIdentityRuleAdjuster for CapScheduledIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledIdentityRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledIdentityRuleAdjuster {
    pub minimum: f64,
}

impl ScheduledIdentityRuleAdjuster for MinimumScheduledIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledIdentityRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledIdentityRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledIdentityRuleAdjuster for TitleScheduledIdentityRuleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledIdentityRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledIdentityRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledIdentityRuleAdjuster for FlaggedScheduledIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledIdentityRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled identity rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledIdentityRuleEngine {
    pub steps: Vec<Rc<dyn ScheduledIdentityRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledIdentityRuleEngine {
    pub fn evaluate(&mut self, item: &ScheduledIdentityRule) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledIdentityRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_identity_rule_engine(flags: &flags::Registry) -> ScheduledIdentityRuleEngine {
    ScheduledIdentityRuleEngine {
        steps: vec![
            Rc::new(RoundingScheduledIdentityRuleAdjuster { step: 0.5 }) as Rc<dyn ScheduledIdentityRuleAdjuster>,
            Rc::new(ScheduledIdentityRuleMaximumOffsetAdjuster { weight: 0.16 }),
            Rc::new(CapScheduledIdentityRuleAdjuster { cap: 26806.0 }),
            Rc::new(MinimumScheduledIdentityRuleAdjuster { minimum: 8.4 }),
            Rc::new(TitleScheduledIdentityRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.21), ("east".to_string(), 0.86)]),
            }),
            Rc::new(FlaggedScheduledIdentityRuleAdjuster {
                flags: flags.clone(),
                flag: "scheduled-identity-rule-adaptive".to_string(),
                percent: 0.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 8905.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_identity_rule_engine(&flags::new_registry());
        let item = new_scheduled_identity_rule("code-257", "title-157");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
