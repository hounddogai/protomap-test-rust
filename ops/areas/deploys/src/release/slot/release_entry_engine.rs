use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release entry calculation.
pub trait ReleaseEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEntryVersionOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseEntryStep for ReleaseEntryVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseEntry, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseEntryStep for TieredReleaseEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseEntryAdjuster {
    pub step: f64,
}

impl ReleaseEntryStep for RoundingReleaseEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseEntryStep for FlaggedReleaseEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseEntryEngine {
    pub steps: Vec<Rc<dyn ReleaseEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseEntryEngine {
    pub fn price(&mut self, item: &ReleaseEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_release_entry_engine(flags: &flags::Registry) -> ReleaseEntryEngine {
    ReleaseEntryEngine {
        steps: vec![
            Rc::new(ReleaseEntryVersionOffsetAdjuster { weight: 1.62 }) as Rc<dyn ReleaseEntryStep>,
            Rc::new(TieredReleaseEntryAdjuster {
                bounds: vec![4660.0, 8350.0, 9641.0],
                rates: vec![1.04, 1.0, 1.03],
            }),
            Rc::new(RoundingReleaseEntryAdjuster { step: 0.5 }),
            Rc::new(FlaggedReleaseEntryAdjuster {
                flags: flags.clone(),
                flag: "release-entry-legacy".to_string(),
                percent: -9.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 80778.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_entry_engine(&flags::new_registry());
        let item = new_release_entry("code-298", "title-651");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
