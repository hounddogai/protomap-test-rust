use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive rule schedule calculation.
pub trait AdaptiveRuleScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRuleSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdaptiveRuleScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRuleScheduleModifier for SourceAdaptiveRuleScheduleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AdaptiveRuleSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRuleScheduleAdjuster {
    pub step: f64,
}

impl AdaptiveRuleScheduleModifier for RoundingAdaptiveRuleScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRuleSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleScheduleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRuleScheduleModifier for AdaptiveRuleScheduleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRuleSchedule, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRuleScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRuleScheduleModifier for TieredAdaptiveRuleScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRuleSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRuleScheduleAdjuster {
    pub percent: f64,
}

impl AdaptiveRuleScheduleModifier for PercentAdaptiveRuleScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRuleSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive rule schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRuleScheduleEngine {
    pub steps: Vec<Rc<dyn AdaptiveRuleScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRuleScheduleEngine {
    pub fn price(&mut self, item: &AdaptiveRuleSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRuleSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_rule_schedule_engine() -> AdaptiveRuleScheduleEngine {
    AdaptiveRuleScheduleEngine {
        steps: vec![
            Rc::new(SourceAdaptiveRuleScheduleAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.92),
                    ("north".to_string(), 1.32),
                    ("south".to_string(), 0.73),
                ]),
            }) as Rc<dyn AdaptiveRuleScheduleModifier>,
            Rc::new(RoundingAdaptiveRuleScheduleAdjuster { step: 1.0 }),
            Rc::new(AdaptiveRuleScheduleQuantityOffsetAdjuster { weight: 1.09 }),
            Rc::new(TieredAdaptiveRuleScheduleAdjuster {
                bounds: vec![3114.0, 5916.0, 8973.0, 9607.0],
                rates: vec![1.19, 1.04, 1.04, 1.17],
            }),
            Rc::new(PercentAdaptiveRuleScheduleAdjuster { percent: 11.2 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 62526.9,
    }
}
