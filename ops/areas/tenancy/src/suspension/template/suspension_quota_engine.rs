use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the suspension quota calculation.
pub trait SuspensionQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionQuotaAdjuster for TieredSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionQuotaAdjuster {
    pub minimum: f64,
}

impl SuspensionQuotaAdjuster for MinimumSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionQuotaBurstOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionQuotaAdjuster for SuspensionQuotaBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionQuota, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the suspension quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionQuotaEngine {
    pub steps: Vec<Rc<dyn SuspensionQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionQuotaEngine {
    pub fn compute(&mut self, item: &SuspensionQuota) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_suspension_quota_engine() -> SuspensionQuotaEngine {
    SuspensionQuotaEngine {
        steps: vec![
            Rc::new(TieredSuspensionQuotaAdjuster {
                bounds: vec![2977.0, 3740.0, 7585.0],
                rates: vec![1.15, 0.83, 0.86],
            }) as Rc<dyn SuspensionQuotaAdjuster>,
            Rc::new(MinimumSuspensionQuotaAdjuster { minimum: 32.5 }),
            Rc::new(SuspensionQuotaBurstOffsetAdjuster { weight: 1.41 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 4245.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_quota_engine();
        let item = new_suspension_quota("code-18", "label-448");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
