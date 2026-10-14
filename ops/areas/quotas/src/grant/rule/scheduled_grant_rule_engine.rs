use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled grant rule calculation.
pub trait ScheduledGrantRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledGrantRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledGrantRuleAdjuster {
    pub percent: f64,
}

impl ScheduledGrantRuleStep for PercentScheduledGrantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledGrantRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledGrantRuleAdjuster {
    pub cap: f64,
}

impl ScheduledGrantRuleStep for CapScheduledGrantRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledGrantRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledGrantRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledGrantRuleStep for FlaggedScheduledGrantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledGrantRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledGrantRuleAdjuster {
    pub step: f64,
}

impl ScheduledGrantRuleStep for RoundingScheduledGrantRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledGrantRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledGrantRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledGrantRuleStep for TieredScheduledGrantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledGrantRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledGrantRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledGrantRuleStep for NameScheduledGrantRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledGrantRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled grant rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledGrantRuleEngine {
    pub steps: Vec<Rc<dyn ScheduledGrantRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledGrantRuleEngine {
    pub fn run(&mut self, item: &ScheduledGrantRule) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledGrantRuleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledGrantRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_grant_rule_engine(flags: &flags::Registry) -> ScheduledGrantRuleEngine {
    ScheduledGrantRuleEngine {
        steps: vec![
            Rc::new(PercentScheduledGrantRuleAdjuster { percent: -15.1 }) as Rc<dyn ScheduledGrantRuleStep>,
            Rc::new(CapScheduledGrantRuleAdjuster { cap: 33476.0 }),
            Rc::new(FlaggedScheduledGrantRuleAdjuster {
                flags: flags.clone(),
                flag: "scheduled-grant-rule-legacy".to_string(),
                percent: -2.1,
            }),
            Rc::new(RoundingScheduledGrantRuleAdjuster { step: 1.0 }),
            Rc::new(TieredScheduledGrantRuleAdjuster {
                bounds: vec![26.0, 4626.0, 6682.0, 9401.0],
                rates: vec![1.08, 1.03, 1.06, 1.18],
            }),
            Rc::new(NameScheduledGrantRuleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.05)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 23856.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_grant_rule_engine(&flags::new_registry());
        let item = new_scheduled_grant_rule("code-186", "name-840");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
