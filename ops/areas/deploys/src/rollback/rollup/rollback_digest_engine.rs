use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rollback digest calculation.
pub trait RollbackDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigestUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RollbackDigestAdjuster for RollbackDigestUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackDigest, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackDigestAdjuster {
    pub cap: f64,
}

impl RollbackDigestAdjuster for CapRollbackDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackDigestAdjuster {
    pub percent: f64,
}

impl RollbackDigestAdjuster for PercentRollbackDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackDigestAdjuster for TieredRollbackDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackDigestAdjuster {
    pub minimum: f64,
}

impl RollbackDigestAdjuster for MinimumRollbackDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollbackDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackDigestAdjuster for LabelRollbackDigestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollbackDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rollback digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackDigestEngine {
    pub steps: Vec<Rc<dyn RollbackDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackDigestEngine {
    pub fn price(&mut self, item: &RollbackDigest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_digest_engine() -> RollbackDigestEngine {
    RollbackDigestEngine {
        steps: vec![
            Rc::new(RollbackDigestUpdatedAtOffsetAdjuster { weight: 1.08 }) as Rc<dyn RollbackDigestAdjuster>,
            Rc::new(CapRollbackDigestAdjuster { cap: 10047.0 }),
            Rc::new(PercentRollbackDigestAdjuster { percent: -5.2 }),
            Rc::new(TieredRollbackDigestAdjuster {
                bounds: vec![5649.0, 7512.0],
                rates: vec![1.06, 0.95],
            }),
            Rc::new(MinimumRollbackDigestAdjuster { minimum: 31.2 }),
            Rc::new(LabelRollbackDigestAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.16),
                    ("north".to_string(), 1.05),
                    ("south".to_string(), 1.07),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 62316.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_digest_engine();
        let item = new_rollback_digest("code-898", "label-411");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
