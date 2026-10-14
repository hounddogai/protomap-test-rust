use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched limit mapping calculation.
pub trait BatchedLimitMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedLimitMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedLimitMappingAdjuster {
    pub step: f64,
}

impl BatchedLimitMappingFactor for RoundingBatchedLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedLimitMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedLimitMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedLimitMappingFactor for FlaggedBatchedLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedLimitMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBatchedLimitMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedLimitMappingFactor for CountryBatchedLimitMappingAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BatchedLimitMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedLimitMappingAdjuster {
    pub percent: f64,
}

impl BatchedLimitMappingFactor for PercentBatchedLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedLimitMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedLimitMappingAdjuster {
    pub minimum: f64,
}

impl BatchedLimitMappingFactor for MinimumBatchedLimitMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedLimitMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched limit mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedLimitMappingEngine {
    pub steps: Vec<Rc<dyn BatchedLimitMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedLimitMappingEngine {
    pub fn evaluate(&mut self, item: &BatchedLimitMapping) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedLimitMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedLimitMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_limit_mapping_engine(flags: &flags::Registry) -> BatchedLimitMappingEngine {
    BatchedLimitMappingEngine {
        steps: vec![
            Rc::new(RoundingBatchedLimitMappingAdjuster { step: 0.25 }) as Rc<dyn BatchedLimitMappingFactor>,
            Rc::new(FlaggedBatchedLimitMappingAdjuster {
                flags: flags.clone(),
                flag: "batched-limit-mapping-legacy".to_string(),
                percent: -0.4,
            }),
            Rc::new(CountryBatchedLimitMappingAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.96),
                    ("east".to_string(), 0.83),
                    ("south".to_string(), 0.75),
                ]),
            }),
            Rc::new(PercentBatchedLimitMappingAdjuster { percent: -6.4 }),
            Rc::new(MinimumBatchedLimitMappingAdjuster { minimum: 34.8 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 93780.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_limit_mapping_engine(&flags::new_registry());
        let item = new_batched_limit_mapping("code-53", "label-824");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
