use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release draft calculation.
pub trait ReleaseDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseDraftAdjuster {
    pub minimum: f64,
}

impl ReleaseDraftAdjuster for MinimumReleaseDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDraftBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseDraftAdjuster for ReleaseDraftBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseDraft, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseDraftAdjuster {
    pub step: f64,
}

impl ReleaseDraftAdjuster for RoundingReleaseDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseDraftAdjuster for FlaggedReleaseDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseDraftAdjuster {
    pub cap: f64,
}

impl ReleaseDraftAdjuster for CapReleaseDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseDraftAdjuster for TieredReleaseDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the release draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseDraftEngine {
    pub steps: Vec<Rc<dyn ReleaseDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseDraftEngine {
    pub fn price(&mut self, item: &ReleaseDraft) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseDraftAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_release_draft_engine(flags: &flags::Registry) -> ReleaseDraftEngine {
    ReleaseDraftEngine {
        steps: vec![
            Rc::new(MinimumReleaseDraftAdjuster { minimum: 26.5 }) as Rc<dyn ReleaseDraftAdjuster>,
            Rc::new(ReleaseDraftBatchSizeOffsetAdjuster { weight: 1.32 }),
            Rc::new(RoundingReleaseDraftAdjuster { step: 0.5 }),
            Rc::new(FlaggedReleaseDraftAdjuster {
                flags: flags.clone(),
                flag: "release-draft-tiered".to_string(),
                percent: 9.3,
            }),
            Rc::new(CapReleaseDraftAdjuster { cap: 49087.0 }),
            Rc::new(TieredReleaseDraftAdjuster {
                bounds: vec![5361.0, 6464.0, 8800.0],
                rates: vec![0.92, 0.93, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 15343.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_draft_engine(&flags::new_registry());
        let item = new_release_draft("code-569", "name-383");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
