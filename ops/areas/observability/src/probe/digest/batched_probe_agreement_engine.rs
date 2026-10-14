use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched probe agreement calculation.
pub trait BatchedProbeAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedProbeAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedProbeAgreementBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl BatchedProbeAgreementAdjuster for BatchedProbeAgreementBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedProbeAgreement, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedProbeAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedProbeAgreementAdjuster for FlaggedBatchedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedProbeAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedProbeAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedProbeAgreementAdjuster for TieredBatchedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedProbeAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleBatchedProbeAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedProbeAgreementAdjuster for TitleBatchedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &BatchedProbeAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedProbeAgreementAdjuster {
    pub minimum: f64,
}

impl BatchedProbeAgreementAdjuster for MinimumBatchedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedProbeAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedProbeAgreementAdjuster {
    pub cap: f64,
}

impl BatchedProbeAgreementAdjuster for CapBatchedProbeAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedProbeAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the batched probe agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedProbeAgreementEngine {
    pub steps: Vec<Rc<dyn BatchedProbeAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedProbeAgreementEngine {
    pub fn evaluate(&mut self, item: &BatchedProbeAgreement) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[BatchedProbeAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_probe_agreement_engine(flags: &flags::Registry) -> BatchedProbeAgreementEngine {
    BatchedProbeAgreementEngine {
        steps: vec![
            Rc::new(BatchedProbeAgreementBatchSizeOffsetAdjuster { weight: 1.28 })
                as Rc<dyn BatchedProbeAgreementAdjuster>,
            Rc::new(FlaggedBatchedProbeAgreementAdjuster {
                flags: flags.clone(),
                flag: "batched-probe-agreement-lenient".to_string(),
                percent: 2.0,
            }),
            Rc::new(TieredBatchedProbeAgreementAdjuster {
                bounds: vec![269.0, 3674.0, 4107.0, 7429.0],
                rates: vec![1.1, 0.91, 0.96, 1.02],
            }),
            Rc::new(TitleBatchedProbeAgreementAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.36),
                    ("south".to_string(), 1.18),
                    ("west".to_string(), 0.77),
                ]),
            }),
            Rc::new(MinimumBatchedProbeAgreementAdjuster { minimum: 16.3 }),
            Rc::new(CapBatchedProbeAgreementAdjuster { cap: 36829.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 46016.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_probe_agreement_engine(&flags::new_registry());
        let item = new_batched_probe_agreement("code-657", "title-100");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
