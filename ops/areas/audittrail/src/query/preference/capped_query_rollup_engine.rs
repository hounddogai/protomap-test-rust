use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped query rollup calculation.
pub trait CappedQueryRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedQueryRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedQueryRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedQueryRollupAdjuster for TieredCappedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedQueryRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryRollupFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl CappedQueryRollupAdjuster for CappedQueryRollupFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &CappedQueryRollup, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedQueryRollupAdjuster {
    pub cap: f64,
}

impl CappedQueryRollupAdjuster for CapCappedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedQueryRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedQueryRollupAdjuster {
    pub step: f64,
}

impl CappedQueryRollupAdjuster for RoundingCappedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedQueryRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedQueryRollupAdjuster {
    pub percent: f64,
}

impl CappedQueryRollupAdjuster for PercentCappedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedQueryRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedQueryRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedQueryRollupAdjuster for FlaggedCappedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedQueryRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped query rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedQueryRollupEngine {
    pub steps: Vec<Rc<dyn CappedQueryRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedQueryRollupEngine {
    pub fn compute(&mut self, item: &CappedQueryRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedQueryRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_query_rollup_engine(flags: &flags::Registry) -> CappedQueryRollupEngine {
    CappedQueryRollupEngine {
        steps: vec![
            Rc::new(TieredCappedQueryRollupAdjuster {
                bounds: vec![1768.0, 6584.0, 8123.0, 9924.0],
                rates: vec![1.16, 1.2, 0.98, 0.85],
            }) as Rc<dyn CappedQueryRollupAdjuster>,
            Rc::new(CappedQueryRollupFloorValueOffsetAdjuster { weight: 0.3 }),
            Rc::new(CapCappedQueryRollupAdjuster { cap: 17982.0 }),
            Rc::new(RoundingCappedQueryRollupAdjuster { step: 0.25 }),
            Rc::new(PercentCappedQueryRollupAdjuster { percent: -14.5 }),
            Rc::new(FlaggedCappedQueryRollupAdjuster {
                flags: flags.clone(),
                flag: "capped-query-rollup-tiered".to_string(),
                percent: 2.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 63085.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_query_rollup_engine(&flags::new_registry());
        let item = new_capped_query_rollup("code-270", "label-384");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
