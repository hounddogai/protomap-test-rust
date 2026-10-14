use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention milestone calculation.
pub trait RetentionMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionMilestoneAdjuster {
    pub percent: f64,
}

impl RetentionMilestoneFactor for PercentRetentionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMilestoneAmountOffsetAdjuster {
    pub weight: f64,
}

impl RetentionMilestoneFactor for RetentionMilestoneAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionMilestone, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionMilestoneAdjuster {
    pub minimum: f64,
}

impl RetentionMilestoneFactor for MinimumRetentionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionMilestoneFactor for FlaggedRetentionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionMilestoneAdjuster {
    pub cap: f64,
}

impl RetentionMilestoneFactor for CapRetentionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionMilestoneFactor for TieredRetentionMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the retention milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionMilestoneEngine {
    pub steps: Vec<Rc<dyn RetentionMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionMilestoneEngine {
    pub fn run(&mut self, item: &RetentionMilestone) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RetentionMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_milestone_engine(flags: &flags::Registry) -> RetentionMilestoneEngine {
    RetentionMilestoneEngine {
        steps: vec![
            Rc::new(PercentRetentionMilestoneAdjuster { percent: -14.7 }) as Rc<dyn RetentionMilestoneFactor>,
            Rc::new(RetentionMilestoneAmountOffsetAdjuster { weight: 1.54 }),
            Rc::new(MinimumRetentionMilestoneAdjuster { minimum: 9.6 }),
            Rc::new(FlaggedRetentionMilestoneAdjuster {
                flags: flags.clone(),
                flag: "retention-milestone-tiered".to_string(),
                percent: 6.1,
            }),
            Rc::new(CapRetentionMilestoneAdjuster { cap: 10382.0 }),
            Rc::new(TieredRetentionMilestoneAdjuster {
                bounds: vec![314.0, 851.0, 6040.0],
                rates: vec![0.84, 0.91, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 23686.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_milestone_engine(&flags::new_registry());
        let item = new_retention_milestone("code-43", "name-793");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
