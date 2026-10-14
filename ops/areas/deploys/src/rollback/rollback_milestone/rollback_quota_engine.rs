use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rollback quota calculation.
pub trait RollbackQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackQuotaVersionOffsetAdjuster {
    pub weight: f64,
}

impl RollbackQuotaFactor for RollbackQuotaVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackQuota, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackQuotaFactor for FlaggedRollbackQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackQuotaAdjuster {
    pub minimum: f64,
}

impl RollbackQuotaFactor for MinimumRollbackQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rollback quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackQuotaEngine {
    pub steps: Vec<Rc<dyn RollbackQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackQuotaEngine {
    pub fn compute(&mut self, item: &RollbackQuota) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_quota_engine(flags: &flags::Registry) -> RollbackQuotaEngine {
    RollbackQuotaEngine {
        steps: vec![
            Rc::new(RollbackQuotaVersionOffsetAdjuster { weight: 1.46 }) as Rc<dyn RollbackQuotaFactor>,
            Rc::new(FlaggedRollbackQuotaAdjuster {
                flags: flags.clone(),
                flag: "rollback-quota-rolling".to_string(),
                percent: -6.2,
            }),
            Rc::new(MinimumRollbackQuotaAdjuster { minimum: 2.6 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 32338.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_quota_engine(&flags::new_registry());
        let item = new_rollback_quota("code-530", "label-972");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
