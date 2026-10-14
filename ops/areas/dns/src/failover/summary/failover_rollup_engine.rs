use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover rollup calculation.
pub trait FailoverRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverRollupStep for TieredFailoverRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverRollupAdjuster {
    pub cap: f64,
}

impl FailoverRollupStep for CapFailoverRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFailoverRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverRollupStep for LabelFailoverRollupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FailoverRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRollupQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl FailoverRollupStep for FailoverRollupQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverRollup, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the failover rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverRollupEngine {
    pub steps: Vec<Rc<dyn FailoverRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverRollupEngine {
    pub fn price(&mut self, item: &FailoverRollup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[FailoverRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_rollup_engine() -> FailoverRollupEngine {
    FailoverRollupEngine {
        steps: vec![
            Rc::new(TieredFailoverRollupAdjuster {
                bounds: vec![3063.0, 4424.0, 9949.0],
                rates: vec![0.9, 1.03, 1.2],
            }) as Rc<dyn FailoverRollupStep>,
            Rc::new(CapFailoverRollupAdjuster { cap: 28177.0 }),
            Rc::new(LabelFailoverRollupAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.88), ("west".to_string(), 0.86)]),
            }),
            Rc::new(FailoverRollupQuotaUnitsOffsetAdjuster { weight: 1.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 19547.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_rollup_engine();
        let item = new_failover_rollup("code-57", "label-103");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
