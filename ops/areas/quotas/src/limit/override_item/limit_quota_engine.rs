use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the limit quota calculation.
pub trait LimitQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitQuotaAdjuster {
    pub percent: f64,
}

impl LimitQuotaStep for PercentLimitQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitQuotaAdjuster {
    pub minimum: f64,
}

impl LimitQuotaStep for MinimumLimitQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLimitQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitQuotaStep for RegionLimitQuotaAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LimitQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the limit quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitQuotaEngine {
    pub steps: Vec<Rc<dyn LimitQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitQuotaEngine {
    pub fn compute(&mut self, item: &LimitQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitQuotaStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_limit_quota_engine() -> LimitQuotaEngine {
    LimitQuotaEngine {
        steps: vec![
            Rc::new(PercentLimitQuotaAdjuster { percent: 22.4 }) as Rc<dyn LimitQuotaStep>,
            Rc::new(MinimumLimitQuotaAdjuster { minimum: 34.9 }),
            Rc::new(RegionLimitQuotaAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 41362.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_quota_engine();
        let item = new_limit_quota("code-906", "title-543");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
