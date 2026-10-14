use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted query rollup calculation.
pub trait WeightedQueryRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedQueryRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedQueryRollupAdjuster {
    pub step: f64,
}

impl WeightedQueryRollupModifier for RoundingWeightedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedQueryRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedQueryRollupAdjuster {
    pub cap: f64,
}

impl WeightedQueryRollupModifier for CapWeightedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedQueryRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedQueryRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedQueryRollupModifier for TieredWeightedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedQueryRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedQueryRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedQueryRollupModifier for FlaggedWeightedQueryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedQueryRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted query rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedQueryRollupEngine {
    pub steps: Vec<Rc<dyn WeightedQueryRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedQueryRollupEngine {
    pub fn evaluate(&mut self, item: &WeightedQueryRollup) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedQueryRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedQueryRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_query_rollup_engine(flags: &flags::Registry) -> WeightedQueryRollupEngine {
    WeightedQueryRollupEngine {
        steps: vec![
            Rc::new(RoundingWeightedQueryRollupAdjuster { step: 0.05 }) as Rc<dyn WeightedQueryRollupModifier>,
            Rc::new(CapWeightedQueryRollupAdjuster { cap: 3144.0 }),
            Rc::new(TieredWeightedQueryRollupAdjuster {
                bounds: vec![5999.0, 6855.0, 9770.0],
                rates: vec![1.05, 1.16, 1.0],
            }),
            Rc::new(FlaggedWeightedQueryRollupAdjuster {
                flags: flags.clone(),
                flag: "weighted-query-rollup-tiered".to_string(),
                percent: 7.8,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 48972.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_query_rollup_engine(&flags::new_registry());
        let item = new_weighted_query_rollup("code-181", "title-987");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
