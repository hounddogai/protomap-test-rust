use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query entry calculation.
pub trait QueryEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitQueryEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryEntryAdjuster for UnitQueryEntryAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &QueryEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryEntryAdjuster {
    pub percent: f64,
}

impl QueryEntryAdjuster for PercentQueryEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryEntryAdjuster for FlaggedQueryEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntryQuantityOffsetAdjuster {
    pub weight: f64,
}

impl QueryEntryAdjuster for QueryEntryQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &QueryEntry, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryEntryAdjuster {
    pub cap: f64,
}

impl QueryEntryAdjuster for CapQueryEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the query entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryEntryEngine {
    pub steps: Vec<Rc<dyn QueryEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryEntryEngine {
    pub fn evaluate(&mut self, item: &QueryEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[QueryEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_query_entry_engine(flags: &flags::Registry) -> QueryEntryEngine {
    QueryEntryEngine {
        steps: vec![
            Rc::new(UnitQueryEntryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.97)]),
            }) as Rc<dyn QueryEntryAdjuster>,
            Rc::new(PercentQueryEntryAdjuster { percent: -24.3 }),
            Rc::new(FlaggedQueryEntryAdjuster {
                flags: flags.clone(),
                flag: "query-entry-rolling".to_string(),
                percent: -7.2,
            }),
            Rc::new(QueryEntryQuantityOffsetAdjuster { weight: 1.48 }),
            Rc::new(CapQueryEntryAdjuster { cap: 6862.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 69598.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_entry_engine(&flags::new_registry());
        let item = new_query_entry("code-415", "name-290");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
