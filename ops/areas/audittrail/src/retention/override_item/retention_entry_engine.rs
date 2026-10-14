use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention entry calculation.
pub trait RetentionEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRetentionEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionEntryStep for RegionRetentionEntryAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RetentionEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionEntryAdjuster {
    pub cap: f64,
}

impl RetentionEntryStep for CapRetentionEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionEntryAdjuster {
    pub minimum: f64,
}

impl RetentionEntryStep for MinimumRetentionEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionEntryAdjuster {
    pub step: f64,
}

impl RetentionEntryStep for RoundingRetentionEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionEntryStep for FlaggedRetentionEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionEntryAdjuster {
    pub percent: f64,
}

impl RetentionEntryStep for PercentRetentionEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the retention entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionEntryEngine {
    pub steps: Vec<Rc<dyn RetentionEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionEntryEngine {
    pub fn compute(&mut self, item: &RetentionEntry) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_retention_entry_engine(flags: &flags::Registry) -> RetentionEntryEngine {
    RetentionEntryEngine {
        steps: vec![
            Rc::new(RegionRetentionEntryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.72)]),
            }) as Rc<dyn RetentionEntryStep>,
            Rc::new(CapRetentionEntryAdjuster { cap: 20582.0 }),
            Rc::new(MinimumRetentionEntryAdjuster { minimum: 19.4 }),
            Rc::new(RoundingRetentionEntryAdjuster { step: 0.5 }),
            Rc::new(FlaggedRetentionEntryAdjuster {
                flags: flags.clone(),
                flag: "retention-entry-strict".to_string(),
                percent: 1.0,
            }),
            Rc::new(PercentRetentionEntryAdjuster { percent: -5.8 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 72695.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_entry_engine(&flags::new_registry());
        let item = new_retention_entry("code-145", "name-73");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
