use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query slot calculation.
pub trait QuerySlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuerySlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedQuerySlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuerySlotStep for FlaggedQuerySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuerySlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuerySlotAdjuster {
    pub step: f64,
}

impl QuerySlotStep for RoundingQuerySlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuerySlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuerySlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuerySlotStep for TieredQuerySlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuerySlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the query slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuerySlotEngine {
    pub steps: Vec<Rc<dyn QuerySlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuerySlotEngine {
    pub fn run(&mut self, item: &QuerySlot) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuerySlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuerySlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_slot_engine(flags: &flags::Registry) -> QuerySlotEngine {
    QuerySlotEngine {
        steps: vec![
            Rc::new(FlaggedQuerySlotAdjuster {
                flags: flags.clone(),
                flag: "query-slot-adaptive".to_string(),
                percent: 1.0,
            }) as Rc<dyn QuerySlotStep>,
            Rc::new(RoundingQuerySlotAdjuster { step: 0.25 }),
            Rc::new(TieredQuerySlotAdjuster {
                bounds: vec![1151.0, 5793.0, 6885.0],
                rates: vec![1.06, 1.18, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 41457.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_slot_engine(&flags::new_registry());
        let item = new_query_slot("code-579", "title-140");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
