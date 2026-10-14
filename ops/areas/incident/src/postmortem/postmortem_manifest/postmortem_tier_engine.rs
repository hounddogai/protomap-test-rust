use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem tier calculation.
pub trait PostmortemTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemTierAdjuster for TieredPostmortemTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemTierAdjuster for FlaggedPostmortemTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemTierAdjuster {
    pub cap: f64,
}

impl PostmortemTierAdjuster for CapPostmortemTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemTierAdjuster {
    pub minimum: f64,
}

impl PostmortemTierAdjuster for MinimumPostmortemTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPostmortemTierAdjuster {
    pub percent: f64,
}

impl PostmortemTierAdjuster for PercentPostmortemTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PostmortemTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemTierAdjuster {
    pub step: f64,
}

impl PostmortemTierAdjuster for RoundingPostmortemTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the postmortem tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemTierEngine {
    pub steps: Vec<Rc<dyn PostmortemTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemTierEngine {
    pub fn evaluate(&mut self, item: &PostmortemTier) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_postmortem_tier_engine(flags: &flags::Registry) -> PostmortemTierEngine {
    PostmortemTierEngine {
        steps: vec![
            Rc::new(TieredPostmortemTierAdjuster {
                bounds: vec![4064.0, 7264.0],
                rates: vec![0.86, 1.06],
            }) as Rc<dyn PostmortemTierAdjuster>,
            Rc::new(FlaggedPostmortemTierAdjuster {
                flags: flags.clone(),
                flag: "postmortem-tier-pending".to_string(),
                percent: -1.8,
            }),
            Rc::new(CapPostmortemTierAdjuster { cap: 29331.0 }),
            Rc::new(MinimumPostmortemTierAdjuster { minimum: 47.2 }),
            Rc::new(PercentPostmortemTierAdjuster { percent: 10.1 }),
            Rc::new(RoundingPostmortemTierAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 29200.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_tier_engine(&flags::new_registry());
        let item = new_postmortem_tier("code-231", "label-78");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
