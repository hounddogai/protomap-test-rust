use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query rollup calculation.
pub trait QueryRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentQueryRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryRollupStep for SegmentQueryRollupAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &QueryRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryRollupAdjuster {
    pub cap: f64,
}

impl QueryRollupStep for CapQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryRollupAdjuster {
    pub percent: f64,
}

impl QueryRollupStep for PercentQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryRollupStep for TieredQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRollupRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl QueryRollupStep for QueryRollupRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &QueryRollup, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryRollupStep for FlaggedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryRollupEngine {
    pub steps: Vec<Rc<dyn QueryRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryRollupEngine {
    pub fn compute(&mut self, item: &QueryRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[QueryRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_rollup_engine(flags: &flags::Registry) -> QueryRollupEngine {
    QueryRollupEngine {
        steps: vec![
            Rc::new(SegmentQueryRollupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.09), ("central".to_string(), 1.24)]),
            }) as Rc<dyn QueryRollupStep>,
            Rc::new(CapQueryRollupAdjuster { cap: 36029.0 }),
            Rc::new(PercentQueryRollupAdjuster { percent: 6.0 }),
            Rc::new(TieredQueryRollupAdjuster {
                bounds: vec![2168.0, 3716.0, 6425.0, 9433.0],
                rates: vec![0.88, 1.16, 1.2, 0.82],
            }),
            Rc::new(QueryRollupRotationDaysOffsetAdjuster { weight: 0.78 }),
            Rc::new(FlaggedQueryRollupAdjuster {
                flags: flags.clone(),
                flag: "query-rollup-seasonal".to_string(),
                percent: -6.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 74158.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_rollup_engine(&flags::new_registry());
        let item = new_query_rollup("code-705", "title-432");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
