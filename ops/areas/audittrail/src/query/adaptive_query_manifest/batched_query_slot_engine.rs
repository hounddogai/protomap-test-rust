use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched query slot calculation.
pub trait BatchedQuerySlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedQuerySlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedQuerySlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedQuerySlotFactor for FlaggedBatchedQuerySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedQuerySlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBatchedQuerySlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedQuerySlotFactor for LocaleBatchedQuerySlotAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BatchedQuerySlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedQuerySlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedQuerySlotFactor for TieredBatchedQuerySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedQuerySlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuerySlotBudgetOffsetAdjuster {
    pub weight: f64,
}

impl BatchedQuerySlotFactor for BatchedQuerySlotBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedQuerySlot, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedQuerySlotAdjuster {
    pub minimum: f64,
}

impl BatchedQuerySlotFactor for MinimumBatchedQuerySlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedQuerySlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched query slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedQuerySlotEngine {
    pub steps: Vec<Rc<dyn BatchedQuerySlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedQuerySlotEngine {
    pub fn price(&mut self, item: &BatchedQuerySlot) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BatchedQuerySlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_query_slot_engine(flags: &flags::Registry) -> BatchedQuerySlotEngine {
    BatchedQuerySlotEngine {
        steps: vec![
            Rc::new(FlaggedBatchedQuerySlotAdjuster {
                flags: flags.clone(),
                flag: "batched-query-slot-lenient".to_string(),
                percent: 6.2,
            }) as Rc<dyn BatchedQuerySlotFactor>,
            Rc::new(LocaleBatchedQuerySlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.76)]),
            }),
            Rc::new(TieredBatchedQuerySlotAdjuster {
                bounds: vec![914.0, 1305.0],
                rates: vec![0.89, 1.12],
            }),
            Rc::new(BatchedQuerySlotBudgetOffsetAdjuster { weight: 0.48 }),
            Rc::new(MinimumBatchedQuerySlotAdjuster { minimum: 48.1 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 30163.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_query_slot_engine(&flags::new_registry());
        let item = new_batched_query_slot("code-78", "title-361");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
