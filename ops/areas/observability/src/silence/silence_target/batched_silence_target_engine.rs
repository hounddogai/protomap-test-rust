use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched silence target calculation.
pub trait BatchedSilenceTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSilenceTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedSilenceTargetAdjuster {
    pub percent: f64,
}

impl BatchedSilenceTargetAdjuster for PercentBatchedSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedSilenceTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceTargetRatioOffsetAdjuster {
    pub weight: f64,
}

impl BatchedSilenceTargetAdjuster for BatchedSilenceTargetRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedSilenceTarget, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedSilenceTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedSilenceTargetAdjuster for FlaggedBatchedSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedSilenceTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedSilenceTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedSilenceTargetAdjuster for TieredBatchedSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedSilenceTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedSilenceTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedSilenceTargetAdjuster for NameBatchedSilenceTargetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedSilenceTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched silence target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSilenceTargetEngine {
    pub steps: Vec<Rc<dyn BatchedSilenceTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSilenceTargetEngine {
    pub fn run(&mut self, item: &BatchedSilenceTarget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedSilenceTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_silence_target_engine(flags: &flags::Registry) -> BatchedSilenceTargetEngine {
    BatchedSilenceTargetEngine {
        steps: vec![
            Rc::new(PercentBatchedSilenceTargetAdjuster { percent: -3.1 }) as Rc<dyn BatchedSilenceTargetAdjuster>,
            Rc::new(BatchedSilenceTargetRatioOffsetAdjuster { weight: 0.72 }),
            Rc::new(FlaggedBatchedSilenceTargetAdjuster {
                flags: flags.clone(),
                flag: "batched-silence-target-primary".to_string(),
                percent: 10.0,
            }),
            Rc::new(TieredBatchedSilenceTargetAdjuster {
                bounds: vec![172.0, 4856.0, 7687.0, 8785.0],
                rates: vec![0.85, 1.1, 0.98, 1.1],
            }),
            Rc::new(NameBatchedSilenceTargetAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.96),
                    ("central".to_string(), 0.91),
                    ("retail".to_string(), 1.29),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 62905.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_silence_target_engine(&flags::new_registry());
        let item = new_batched_silence_target("code-340", "name-179");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
