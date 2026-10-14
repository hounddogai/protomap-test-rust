use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched allowlist quota calculation.
pub trait BatchedAllowlistQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAllowlistQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistQuotaBudgetOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAllowlistQuotaModifier for BatchedAllowlistQuotaBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAllowlistQuota, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedAllowlistQuotaAdjuster {
    pub step: f64,
}

impl BatchedAllowlistQuotaModifier for RoundingBatchedAllowlistQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedAllowlistQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedAllowlistQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedAllowlistQuotaModifier for FlaggedBatchedAllowlistQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedAllowlistQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceBatchedAllowlistQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedAllowlistQuotaModifier for NamespaceBatchedAllowlistQuotaAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &BatchedAllowlistQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedAllowlistQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedAllowlistQuotaModifier for TieredBatchedAllowlistQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedAllowlistQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched allowlist quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistQuotaEngine {
    pub steps: Vec<Rc<dyn BatchedAllowlistQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAllowlistQuotaEngine {
    pub fn run(&mut self, item: &BatchedAllowlistQuota) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAllowlistQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_allowlist_quota_engine(flags: &flags::Registry) -> BatchedAllowlistQuotaEngine {
    BatchedAllowlistQuotaEngine {
        steps: vec![
            Rc::new(BatchedAllowlistQuotaBudgetOffsetAdjuster { weight: 0.07 })
                as Rc<dyn BatchedAllowlistQuotaModifier>,
            Rc::new(RoundingBatchedAllowlistQuotaAdjuster { step: 0.5 }),
            Rc::new(FlaggedBatchedAllowlistQuotaAdjuster {
                flags: flags.clone(),
                flag: "batched-allowlist-quota-fallback".to_string(),
                percent: -8.8,
            }),
            Rc::new(NamespaceBatchedAllowlistQuotaAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.9),
                    ("retail".to_string(), 0.84),
                    ("east".to_string(), 0.82),
                ]),
            }),
            Rc::new(TieredBatchedAllowlistQuotaAdjuster {
                bounds: vec![5081.0, 8369.0],
                rates: vec![1.17, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 69284.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_allowlist_quota_engine(&flags::new_registry());
        let item = new_batched_allowlist_quota("code-535", "name-593");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
