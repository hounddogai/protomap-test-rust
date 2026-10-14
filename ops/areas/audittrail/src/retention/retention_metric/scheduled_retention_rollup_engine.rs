use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled retention rollup calculation.
pub trait ScheduledRetentionRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRetentionRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceScheduledRetentionRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledRetentionRollupStep for NamespaceScheduledRetentionRollupAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ScheduledRetentionRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledRetentionRollupAdjuster {
    pub percent: f64,
}

impl ScheduledRetentionRollupStep for PercentScheduledRetentionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledRetentionRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledRetentionRollupAdjuster {
    pub cap: f64,
}

impl ScheduledRetentionRollupStep for CapScheduledRetentionRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledRetentionRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRetentionRollupRatioOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledRetentionRollupStep for ScheduledRetentionRollupRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledRetentionRollup, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledRetentionRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledRetentionRollupStep for TieredScheduledRetentionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledRetentionRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the scheduled retention rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRetentionRollupEngine {
    pub steps: Vec<Rc<dyn ScheduledRetentionRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRetentionRollupEngine {
    pub fn price(&mut self, item: &ScheduledRetentionRollup) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledRetentionRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_retention_rollup_engine() -> ScheduledRetentionRollupEngine {
    ScheduledRetentionRollupEngine {
        steps: vec![
            Rc::new(NamespaceScheduledRetentionRollupAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.07),
                    ("west".to_string(), 0.99),
                    ("central".to_string(), 1.01),
                ]),
            }) as Rc<dyn ScheduledRetentionRollupStep>,
            Rc::new(PercentScheduledRetentionRollupAdjuster { percent: 21.6 }),
            Rc::new(CapScheduledRetentionRollupAdjuster { cap: 35007.0 }),
            Rc::new(ScheduledRetentionRollupRatioOffsetAdjuster { weight: 1.14 }),
            Rc::new(TieredScheduledRetentionRollupAdjuster {
                bounds: vec![4238.0, 4542.0, 6782.0],
                rates: vec![0.93, 1.13, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 75709.4,
    }
}
