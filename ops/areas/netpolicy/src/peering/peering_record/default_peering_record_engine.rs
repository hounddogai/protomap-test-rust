use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default peering record calculation.
pub trait DefaultPeeringRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPeeringRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultPeeringRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultPeeringRecordModifier for TieredDefaultPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultPeeringRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultPeeringRecordAdjuster {
    pub cap: f64,
}

impl DefaultPeeringRecordModifier for CapDefaultPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultPeeringRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDefaultPeeringRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultPeeringRecordModifier for ClusterDefaultPeeringRecordAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DefaultPeeringRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultPeeringRecordAdjuster {
    pub step: f64,
}

impl DefaultPeeringRecordModifier for RoundingDefaultPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultPeeringRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default peering record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPeeringRecordEngine {
    pub steps: Vec<Rc<dyn DefaultPeeringRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPeeringRecordEngine {
    pub fn price(&mut self, item: &DefaultPeeringRecord) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPeeringRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_peering_record_engine() -> DefaultPeeringRecordEngine {
    DefaultPeeringRecordEngine {
        steps: vec![
            Rc::new(TieredDefaultPeeringRecordAdjuster {
                bounds: vec![2013.0, 5777.0],
                rates: vec![1.2, 1.09],
            }) as Rc<dyn DefaultPeeringRecordModifier>,
            Rc::new(CapDefaultPeeringRecordAdjuster { cap: 7275.0 }),
            Rc::new(ClusterDefaultPeeringRecordAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.78)]),
            }),
            Rc::new(RoundingDefaultPeeringRecordAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 73396.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_peering_record_engine();
        let item = new_default_peering_record("code-25", "name-25");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
