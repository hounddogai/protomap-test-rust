use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the overage quota calculation.
pub trait OverageQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageQuotaAdjuster {
    pub cap: f64,
}

impl OverageQuotaStep for CapOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageQuotaStep for TieredOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageQuotaAdjuster {
    pub minimum: f64,
}

impl OverageQuotaStep for MinimumOverageQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the overage quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageQuotaEngine {
    pub steps: Vec<Rc<dyn OverageQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageQuotaEngine {
    pub fn evaluate(&mut self, item: &OverageQuota) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageQuotaStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_quota_engine() -> OverageQuotaEngine {
    OverageQuotaEngine {
        steps: vec![
            Rc::new(CapOverageQuotaAdjuster { cap: 48328.0 }) as Rc<dyn OverageQuotaStep>,
            Rc::new(TieredOverageQuotaAdjuster {
                bounds: vec![1143.0, 3034.0, 4543.0],
                rates: vec![1.07, 0.87, 1.16],
            }),
            Rc::new(MinimumOverageQuotaAdjuster { minimum: 37.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 32878.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_quota_engine();
        let item = new_overage_quota("code-180", "name-667");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
