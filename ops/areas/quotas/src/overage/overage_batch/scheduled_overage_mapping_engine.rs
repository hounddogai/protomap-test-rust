use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled overage mapping calculation.
pub trait ScheduledOverageMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledOverageMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledOverageMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledOverageMappingModifier for TitleScheduledOverageMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledOverageMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageMappingRevisionOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledOverageMappingModifier for ScheduledOverageMappingRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledOverageMapping, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledOverageMappingAdjuster {
    pub step: f64,
}

impl ScheduledOverageMappingModifier for RoundingScheduledOverageMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledOverageMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled overage mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledOverageMappingEngine {
    pub steps: Vec<Rc<dyn ScheduledOverageMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledOverageMappingEngine {
    pub fn price(&mut self, item: &ScheduledOverageMapping) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledOverageMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledOverageMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_overage_mapping_engine() -> ScheduledOverageMappingEngine {
    ScheduledOverageMappingEngine {
        steps: vec![
            Rc::new(TitleScheduledOverageMappingAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.85), ("online".to_string(), 0.72)]),
            }) as Rc<dyn ScheduledOverageMappingModifier>,
            Rc::new(ScheduledOverageMappingRevisionOffsetAdjuster { weight: 1.58 }),
            Rc::new(RoundingScheduledOverageMappingAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 84961.5,
    }
}
