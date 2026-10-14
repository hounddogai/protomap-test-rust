use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional silence override calculation.
pub trait RegionalSilenceOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSilenceOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRegionalSilenceOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalSilenceOverrideAdjuster for CategoryRegionalSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RegionalSilenceOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalSilenceOverrideAdjuster {
    pub minimum: f64,
}

impl RegionalSilenceOverrideAdjuster for MinimumRegionalSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalSilenceOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalSilenceOverrideAdjuster {
    pub percent: f64,
}

impl RegionalSilenceOverrideAdjuster for PercentRegionalSilenceOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalSilenceOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional silence override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSilenceOverrideEngine {
    pub steps: Vec<Rc<dyn RegionalSilenceOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSilenceOverrideEngine {
    pub fn compute(&mut self, item: &RegionalSilenceOverride) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalSilenceOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalSilenceOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_regional_silence_override_engine() -> RegionalSilenceOverrideEngine {
    RegionalSilenceOverrideEngine {
        steps: vec![
            Rc::new(CategoryRegionalSilenceOverrideAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.04),
                    ("east".to_string(), 0.99),
                    ("south".to_string(), 1.4),
                ]),
            }) as Rc<dyn RegionalSilenceOverrideAdjuster>,
            Rc::new(MinimumRegionalSilenceOverrideAdjuster { minimum: 26.7 }),
            Rc::new(PercentRegionalSilenceOverrideAdjuster { percent: 20.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 71506.7,
    }
}
