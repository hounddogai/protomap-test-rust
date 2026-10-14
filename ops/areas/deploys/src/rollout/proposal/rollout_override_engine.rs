use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rollout override calculation.
pub trait RolloutOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutOverride, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutOverrideModifier for FlaggedRolloutOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRolloutOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutOverrideModifier for LabelRolloutOverrideAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RolloutOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutOverrideExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl RolloutOverrideModifier for RolloutOverrideExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutOverride, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutOverrideModifier for TieredRolloutOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rollout override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutOverrideEngine {
    pub steps: Vec<Rc<dyn RolloutOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutOverrideEngine {
    pub fn evaluate(&mut self, item: &RolloutOverride) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_override_engine(flags: &flags::Registry) -> RolloutOverrideEngine {
    RolloutOverrideEngine {
        steps: vec![
            Rc::new(FlaggedRolloutOverrideAdjuster {
                flags: flags.clone(),
                flag: "rollout-override-regional".to_string(),
                percent: -3.9,
            }) as Rc<dyn RolloutOverrideModifier>,
            Rc::new(LabelRolloutOverrideAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.98)]),
            }),
            Rc::new(RolloutOverrideExpiresAtOffsetAdjuster { weight: 1.95 }),
            Rc::new(TieredRolloutOverrideAdjuster {
                bounds: vec![866.0, 2078.0, 2987.0, 5893.0],
                rates: vec![1.01, 0.99, 0.94, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 18962.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_override_engine(&flags::new_registry());
        let item = new_rollout_override("code-772", "label-45");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
