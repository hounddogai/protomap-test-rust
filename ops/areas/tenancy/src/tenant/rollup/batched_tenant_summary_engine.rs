use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched tenant summary calculation.
pub trait BatchedTenantSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTenantSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedTenantSummaryAdjuster {
    pub percent: f64,
}

impl BatchedTenantSummaryAdjuster for PercentBatchedTenantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedTenantSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantSummaryScoreOffsetAdjuster {
    pub weight: f64,
}

impl BatchedTenantSummaryAdjuster for BatchedTenantSummaryScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedTenantSummary, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedTenantSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedTenantSummaryAdjuster for FlaggedBatchedTenantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedTenantSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedTenantSummaryAdjuster {
    pub minimum: f64,
}

impl BatchedTenantSummaryAdjuster for MinimumBatchedTenantSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedTenantSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBatchedTenantSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedTenantSummaryAdjuster for LocaleBatchedTenantSummaryAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BatchedTenantSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched tenant summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTenantSummaryEngine {
    pub steps: Vec<Rc<dyn BatchedTenantSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTenantSummaryEngine {
    pub fn evaluate(&mut self, item: &BatchedTenantSummary) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedTenantSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTenantSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_tenant_summary_engine(flags: &flags::Registry) -> BatchedTenantSummaryEngine {
    BatchedTenantSummaryEngine {
        steps: vec![
            Rc::new(PercentBatchedTenantSummaryAdjuster { percent: -1.7 }) as Rc<dyn BatchedTenantSummaryAdjuster>,
            Rc::new(BatchedTenantSummaryScoreOffsetAdjuster { weight: 0.07 }),
            Rc::new(FlaggedBatchedTenantSummaryAdjuster {
                flags: flags.clone(),
                flag: "batched-tenant-summary-capped".to_string(),
                percent: -0.1,
            }),
            Rc::new(MinimumBatchedTenantSummaryAdjuster { minimum: 17.2 }),
            Rc::new(LocaleBatchedTenantSummaryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.07), ("central".to_string(), 0.83)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 69380.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_tenant_summary_engine(&flags::new_registry());
        let item = new_batched_tenant_summary("code-77", "label-495");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
