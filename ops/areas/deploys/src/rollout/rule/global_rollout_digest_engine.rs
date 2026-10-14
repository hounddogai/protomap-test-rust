use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global rollout digest calculation.
pub trait GlobalRolloutDigestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalRolloutDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalRolloutDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalRolloutDigestStep for TieredGlobalRolloutDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalRolloutDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalRolloutDigestAdjuster {
    pub cap: f64,
}

impl GlobalRolloutDigestStep for CapGlobalRolloutDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalRolloutDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutDigestDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl GlobalRolloutDigestStep for GlobalRolloutDigestDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalRolloutDigest, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalRolloutDigestAdjuster {
    pub step: f64,
}

impl GlobalRolloutDigestStep for RoundingGlobalRolloutDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalRolloutDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryGlobalRolloutDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalRolloutDigestStep for CountryGlobalRolloutDigestAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &GlobalRolloutDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalRolloutDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalRolloutDigestStep for FlaggedGlobalRolloutDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalRolloutDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global rollout digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalRolloutDigestEngine {
    pub steps: Vec<Rc<dyn GlobalRolloutDigestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalRolloutDigestEngine {
    pub fn price(&mut self, item: &GlobalRolloutDigest) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalRolloutDigestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalRolloutDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_rollout_digest_engine(flags: &flags::Registry) -> GlobalRolloutDigestEngine {
    GlobalRolloutDigestEngine {
        steps: vec![
            Rc::new(TieredGlobalRolloutDigestAdjuster {
                bounds: vec![4884.0, 9241.0],
                rates: vec![0.91, 0.81],
            }) as Rc<dyn GlobalRolloutDigestStep>,
            Rc::new(CapGlobalRolloutDigestAdjuster { cap: 7942.0 }),
            Rc::new(GlobalRolloutDigestDurationMinutesOffsetAdjuster { weight: 0.97 }),
            Rc::new(RoundingGlobalRolloutDigestAdjuster { step: 5.0 }),
            Rc::new(CountryGlobalRolloutDigestAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.83),
                    ("online".to_string(), 0.92),
                    ("north".to_string(), 1.03),
                ]),
            }),
            Rc::new(FlaggedGlobalRolloutDigestAdjuster {
                flags: flags.clone(),
                flag: "global-rollout-digest-legacy".to_string(),
                percent: -1.1,
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 16707.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_rollout_digest_engine(&flags::new_registry());
        let item = new_global_rollout_digest("code-449", "name-828");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
