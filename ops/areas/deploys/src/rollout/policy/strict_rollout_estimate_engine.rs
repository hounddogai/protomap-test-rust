use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict rollout estimate calculation.
pub trait StrictRolloutEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRolloutEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimateDiscountOffsetAdjuster {
    pub weight: f64,
}

impl StrictRolloutEstimateAdjuster for StrictRolloutEstimateDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &StrictRolloutEstimate, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterStrictRolloutEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictRolloutEstimateAdjuster for ClusterStrictRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &StrictRolloutEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictRolloutEstimateAdjuster {
    pub percent: f64,
}

impl StrictRolloutEstimateAdjuster for PercentStrictRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictRolloutEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictRolloutEstimateAdjuster {
    pub minimum: f64,
}

impl StrictRolloutEstimateAdjuster for MinimumStrictRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictRolloutEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictRolloutEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictRolloutEstimateAdjuster for TieredStrictRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictRolloutEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictRolloutEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictRolloutEstimateAdjuster for FlaggedStrictRolloutEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictRolloutEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict rollout estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRolloutEstimateEngine {
    pub steps: Vec<Rc<dyn StrictRolloutEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRolloutEstimateEngine {
    pub fn run(&mut self, item: &StrictRolloutEstimate) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictRolloutEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictRolloutEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_rollout_estimate_engine(flags: &flags::Registry) -> StrictRolloutEstimateEngine {
    StrictRolloutEstimateEngine {
        steps: vec![
            Rc::new(StrictRolloutEstimateDiscountOffsetAdjuster { weight: 0.18 })
                as Rc<dyn StrictRolloutEstimateAdjuster>,
            Rc::new(ClusterStrictRolloutEstimateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.33)]),
            }),
            Rc::new(PercentStrictRolloutEstimateAdjuster { percent: -7.3 }),
            Rc::new(MinimumStrictRolloutEstimateAdjuster { minimum: 12.0 }),
            Rc::new(TieredStrictRolloutEstimateAdjuster {
                bounds: vec![5594.0, 7643.0],
                rates: vec![0.87, 0.95],
            }),
            Rc::new(FlaggedStrictRolloutEstimateAdjuster {
                flags: flags.clone(),
                flag: "strict-rollout-estimate-tiered".to_string(),
                percent: -0.4,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 11609.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_rollout_estimate_engine(&flags::new_registry());
        let item = new_strict_rollout_estimate("code-702", "label-332");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
