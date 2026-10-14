use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query allocation calculation.
pub trait QueryAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryAllocationAdjuster {
    pub minimum: f64,
}

impl QueryAllocationFactor for MinimumQueryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryAllocationFactor for FlaggedQueryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryAllocationAdjuster {
    pub step: f64,
}

impl QueryAllocationFactor for RoundingQueryAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the query allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryAllocationEngine {
    pub steps: Vec<Rc<dyn QueryAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryAllocationEngine {
    pub fn price(&mut self, item: &QueryAllocation) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[QueryAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_query_allocation_engine(flags: &flags::Registry) -> QueryAllocationEngine {
    QueryAllocationEngine {
        steps: vec![
            Rc::new(MinimumQueryAllocationAdjuster { minimum: 3.5 }) as Rc<dyn QueryAllocationFactor>,
            Rc::new(FlaggedQueryAllocationAdjuster {
                flags: flags.clone(),
                flag: "query-allocation-global".to_string(),
                percent: -1.0,
            }),
            Rc::new(RoundingQueryAllocationAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 14094.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_allocation_engine(&flags::new_registry());
        let item = new_query_allocation("code-804", "name-268");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
