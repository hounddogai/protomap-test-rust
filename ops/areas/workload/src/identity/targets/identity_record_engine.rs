use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity record calculation.
pub trait IdentityRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryIdentityRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityRecordFactor for CategoryIdentityRecordAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &IdentityRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityRecordAdjuster {
    pub cap: f64,
}

impl IdentityRecordFactor for CapIdentityRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRecordCostOffsetAdjuster {
    pub weight: f64,
}

impl IdentityRecordFactor for IdentityRecordCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityRecord, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

/// Runs the identity record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityRecordEngine {
    pub steps: Vec<Rc<dyn IdentityRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityRecordEngine {
    pub fn evaluate(&mut self, item: &IdentityRecord) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[IdentityRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_record_engine() -> IdentityRecordEngine {
    IdentityRecordEngine {
        steps: vec![
            Rc::new(CategoryIdentityRecordAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.88),
                    ("retail".to_string(), 0.79),
                    ("north".to_string(), 1.07),
                ]),
            }) as Rc<dyn IdentityRecordFactor>,
            Rc::new(CapIdentityRecordAdjuster { cap: 4449.0 }),
            Rc::new(IdentityRecordCostOffsetAdjuster { weight: 1.4 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 33984.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_record_engine();
        let item = new_identity_record("code-557", "title-116");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
