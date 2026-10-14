use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled rule override calculation.
pub trait ScheduledRuleOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRuleOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleScheduledRuleOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledRuleOverrideModifier for LocaleScheduledRuleOverrideAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ScheduledRuleOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverrideGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledRuleOverrideModifier for ScheduledRuleOverrideGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledRuleOverride, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledRuleOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledRuleOverrideModifier for TieredScheduledRuleOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledRuleOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledRuleOverrideAdjuster {
    pub step: f64,
}

impl ScheduledRuleOverrideModifier for RoundingScheduledRuleOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledRuleOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledRuleOverrideAdjuster {
    pub percent: f64,
}

impl ScheduledRuleOverrideModifier for PercentScheduledRuleOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledRuleOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled rule override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRuleOverrideEngine {
    pub steps: Vec<Rc<dyn ScheduledRuleOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRuleOverrideEngine {
    pub fn evaluate(&mut self, item: &ScheduledRuleOverride) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledRuleOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledRuleOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_rule_override_engine() -> ScheduledRuleOverrideEngine {
    ScheduledRuleOverrideEngine {
        steps: vec![
            Rc::new(LocaleScheduledRuleOverrideAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.09), ("online".to_string(), 1.1)]),
            }) as Rc<dyn ScheduledRuleOverrideModifier>,
            Rc::new(ScheduledRuleOverrideGraceDaysOffsetAdjuster { weight: 0.2 }),
            Rc::new(TieredScheduledRuleOverrideAdjuster {
                bounds: vec![1576.0, 9613.0],
                rates: vec![1.0, 1.01],
            }),
            Rc::new(RoundingScheduledRuleOverrideAdjuster { step: 5.0 }),
            Rc::new(PercentScheduledRuleOverrideAdjuster { percent: -10.8 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 15572.0,
    }
}
