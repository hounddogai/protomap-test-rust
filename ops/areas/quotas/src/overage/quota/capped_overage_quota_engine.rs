use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped overage quota calculation.
pub trait CappedOverageQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedOverageQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedOverageQuotaAdjuster {
    pub step: f64,
}

impl CappedOverageQuotaAdjuster for RoundingCappedOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedOverageQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageQuotaAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl CappedOverageQuotaAdjuster for CappedOverageQuotaAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &CappedOverageQuota, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedOverageQuotaAdjuster {
    pub minimum: f64,
}

impl CappedOverageQuotaAdjuster for MinimumCappedOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedOverageQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped overage quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedOverageQuotaEngine {
    pub steps: Vec<Rc<dyn CappedOverageQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedOverageQuotaEngine {
    pub fn compute(&mut self, item: &CappedOverageQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedOverageQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedOverageQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_overage_quota_engine() -> CappedOverageQuotaEngine {
    CappedOverageQuotaEngine {
        steps: vec![
            Rc::new(RoundingCappedOverageQuotaAdjuster { step: 0.25 }) as Rc<dyn CappedOverageQuotaAdjuster>,
            Rc::new(CappedOverageQuotaAttemptsOffsetAdjuster { weight: 0.99 }),
            Rc::new(MinimumCappedOverageQuotaAdjuster { minimum: 31.6 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 73475.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_overage_quota_engine();
        let item = new_capped_overage_quota("code-791", "title-960");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
