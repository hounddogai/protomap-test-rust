use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the redaction profile calculation.
pub trait RedactionProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRedactionProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionProfileFactor for LabelRedactionProfileAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RedactionProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionProfileFactor for TieredRedactionProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionProfileAdjuster {
    pub cap: f64,
}

impl RedactionProfileFactor for CapRedactionProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the redaction profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionProfileEngine {
    pub steps: Vec<Rc<dyn RedactionProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionProfileEngine {
    pub fn price(&mut self, item: &RedactionProfile) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionProfileFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_profile_engine() -> RedactionProfileEngine {
    RedactionProfileEngine {
        steps: vec![
            Rc::new(LabelRedactionProfileAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.88), ("retail".to_string(), 1.24)]),
            }) as Rc<dyn RedactionProfileFactor>,
            Rc::new(TieredRedactionProfileAdjuster {
                bounds: vec![2802.0, 6654.0],
                rates: vec![0.83, 0.82],
            }),
            Rc::new(CapRedactionProfileAdjuster { cap: 47867.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 20345.9,
    }
}
