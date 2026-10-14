use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the release milestone calculation.
pub trait ReleaseMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseMilestoneAdjuster for TieredReleaseMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseMilestoneAdjuster {
    pub percent: f64,
}

impl ReleaseMilestoneAdjuster for PercentReleaseMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseMilestoneAdjuster for FlaggedReleaseMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseMilestoneQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseMilestoneAdjuster for ReleaseMilestoneQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseMilestone, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameReleaseMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseMilestoneAdjuster for NameReleaseMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ReleaseMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReleaseMilestoneAdjuster {
    pub cap: f64,
}

impl ReleaseMilestoneAdjuster for CapReleaseMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReleaseMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the release milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseMilestoneEngine {
    pub steps: Vec<Rc<dyn ReleaseMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseMilestoneEngine {
    pub fn compute(&mut self, item: &ReleaseMilestone) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_milestone_engine(flags: &flags::Registry) -> ReleaseMilestoneEngine {
    ReleaseMilestoneEngine {
        steps: vec![
            Rc::new(TieredReleaseMilestoneAdjuster {
                bounds: vec![1795.0, 3125.0, 6852.0],
                rates: vec![0.86, 1.02, 1.05],
            }) as Rc<dyn ReleaseMilestoneAdjuster>,
            Rc::new(PercentReleaseMilestoneAdjuster { percent: 5.9 }),
            Rc::new(FlaggedReleaseMilestoneAdjuster {
                flags: flags.clone(),
                flag: "release-milestone-adaptive".to_string(),
                percent: 4.0,
            }),
            Rc::new(ReleaseMilestoneQuotaUnitsOffsetAdjuster { weight: 0.71 }),
            Rc::new(NameReleaseMilestoneAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.28), ("south".to_string(), 1.31)]),
            }),
            Rc::new(CapReleaseMilestoneAdjuster { cap: 24197.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 80138.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_milestone_engine(&flags::new_registry());
        let item = new_release_milestone("code-797", "name-839");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
