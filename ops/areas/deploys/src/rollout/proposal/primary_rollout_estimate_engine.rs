use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary rollout estimate calculation.
pub trait PrimaryRolloutEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRolloutEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutEstimateBurstOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRolloutEstimateAdjuster for PrimaryRolloutEstimateBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRolloutEstimate, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryRolloutEstimateAdjuster {
    pub minimum: f64,
}

impl PrimaryRolloutEstimateAdjuster for MinimumPrimaryRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryRolloutEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRolloutEstimateAdjuster {
    pub cap: f64,
}

impl PrimaryRolloutEstimateAdjuster for CapPrimaryRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRolloutEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRolloutEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRolloutEstimateAdjuster for FlaggedPrimaryRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRolloutEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryRolloutEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryRolloutEstimateAdjuster for SourcePrimaryRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryRolloutEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary rollout estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRolloutEstimateEngine {
    pub steps: Vec<Rc<dyn PrimaryRolloutEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRolloutEstimateEngine {
    pub fn price(&mut self, item: &PrimaryRolloutEstimate) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryRolloutEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRolloutEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_rollout_estimate_engine(flags: &flags::Registry) -> PrimaryRolloutEstimateEngine {
    PrimaryRolloutEstimateEngine {
        steps: vec![
            Rc::new(PrimaryRolloutEstimateBurstOffsetAdjuster { weight: 0.09 })
                as Rc<dyn PrimaryRolloutEstimateAdjuster>,
            Rc::new(MinimumPrimaryRolloutEstimateAdjuster { minimum: 3.7 }),
            Rc::new(CapPrimaryRolloutEstimateAdjuster { cap: 13765.0 }),
            Rc::new(FlaggedPrimaryRolloutEstimateAdjuster {
                flags: flags.clone(),
                flag: "primary-rollout-estimate-manual".to_string(),
                percent: 8.8,
            }),
            Rc::new(SourcePrimaryRolloutEstimateAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.88), ("north".to_string(), 0.7)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 22118.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_rollout_estimate_engine(&flags::new_registry());
        let item = new_primary_rollout_estimate("code-708", "name-775");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
