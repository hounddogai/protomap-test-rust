use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive quote group calculation.
pub trait AdaptiveQuoteGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveQuoteGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterAdaptiveQuoteGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveQuoteGroupFactor for ClusterAdaptiveQuoteGroupAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &AdaptiveQuoteGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveQuoteGroupAdjuster {
    pub cap: f64,
}

impl AdaptiveQuoteGroupFactor for CapAdaptiveQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveQuoteGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveQuoteGroupAdjuster {
    pub percent: f64,
}

impl AdaptiveQuoteGroupFactor for PercentAdaptiveQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveQuoteGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive quote group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveQuoteGroupEngine {
    pub steps: Vec<Rc<dyn AdaptiveQuoteGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveQuoteGroupEngine {
    pub fn price(&mut self, item: &AdaptiveQuoteGroup) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveQuoteGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_quote_group_engine() -> AdaptiveQuoteGroupEngine {
    AdaptiveQuoteGroupEngine {
        steps: vec![
            Rc::new(ClusterAdaptiveQuoteGroupAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.13),
                    ("central".to_string(), 0.83),
                    ("south".to_string(), 1.13),
                ]),
            }) as Rc<dyn AdaptiveQuoteGroupFactor>,
            Rc::new(CapAdaptiveQuoteGroupAdjuster { cap: 22421.0 }),
            Rc::new(PercentAdaptiveQuoteGroupAdjuster { percent: -8.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 67600.3,
    }
}
