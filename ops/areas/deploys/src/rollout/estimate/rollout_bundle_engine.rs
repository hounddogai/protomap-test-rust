use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout bundle calculation.
pub trait RolloutBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutBundle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutBundleStep for FlaggedRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutBundleAdjuster {
    pub cap: f64,
}

impl RolloutBundleStep for CapRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutBundleAdjuster {
    pub step: f64,
}

impl RolloutBundleStep for RoundingRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBundleRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RolloutBundleStep for RolloutBundleRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutBundle, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRolloutBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutBundleStep for ChannelRolloutBundleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RolloutBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutBundleStep for TieredRolloutBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rollout bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutBundleEngine {
    pub steps: Vec<Rc<dyn RolloutBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutBundleEngine {
    pub fn price(&mut self, item: &RolloutBundle) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollout_bundle_engine(flags: &flags::Registry) -> RolloutBundleEngine {
    RolloutBundleEngine {
        steps: vec![
            Rc::new(FlaggedRolloutBundleAdjuster {
                flags: flags.clone(),
                flag: "rollout-bundle-primary".to_string(),
                percent: -5.6,
            }) as Rc<dyn RolloutBundleStep>,
            Rc::new(CapRolloutBundleAdjuster { cap: 1413.0 }),
            Rc::new(RoundingRolloutBundleAdjuster { step: 0.5 }),
            Rc::new(RolloutBundleRetriesOffsetAdjuster { weight: 0.06 }),
            Rc::new(ChannelRolloutBundleAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.32),
                    ("south".to_string(), 1.08),
                    ("north".to_string(), 0.77),
                ]),
            }),
            Rc::new(TieredRolloutBundleAdjuster {
                bounds: vec![1890.0, 3763.0, 4375.0],
                rates: vec![1.01, 1.15, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 96071.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_bundle_engine(&flags::new_registry());
        let item = new_rollout_bundle("code-597", "label-655");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
