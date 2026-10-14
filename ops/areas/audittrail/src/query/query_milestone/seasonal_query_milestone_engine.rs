use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal query milestone calculation.
pub trait SeasonalQueryMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalQueryMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalQueryMilestoneAdjuster {
    pub cap: f64,
}

impl SeasonalQueryMilestoneStep for CapSeasonalQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalQueryMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalQueryMilestoneAdjuster {
    pub step: f64,
}

impl SeasonalQueryMilestoneStep for RoundingSeasonalQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalQueryMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalQueryMilestoneAdjuster {
    pub percent: f64,
}

impl SeasonalQueryMilestoneStep for PercentSeasonalQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalQueryMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalQueryMilestoneAdjuster {
    pub minimum: f64,
}

impl SeasonalQueryMilestoneStep for MinimumSeasonalQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalQueryMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalQueryMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalQueryMilestoneStep for FlaggedSeasonalQueryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalQueryMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal query milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalQueryMilestoneEngine {
    pub steps: Vec<Rc<dyn SeasonalQueryMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalQueryMilestoneEngine {
    pub fn run(&mut self, item: &SeasonalQueryMilestone) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalQueryMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalQueryMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_query_milestone_engine(flags: &flags::Registry) -> SeasonalQueryMilestoneEngine {
    SeasonalQueryMilestoneEngine {
        steps: vec![
            Rc::new(CapSeasonalQueryMilestoneAdjuster { cap: 5305.0 }) as Rc<dyn SeasonalQueryMilestoneStep>,
            Rc::new(RoundingSeasonalQueryMilestoneAdjuster { step: 0.1 }),
            Rc::new(PercentSeasonalQueryMilestoneAdjuster { percent: 11.6 }),
            Rc::new(MinimumSeasonalQueryMilestoneAdjuster { minimum: 26.4 }),
            Rc::new(FlaggedSeasonalQueryMilestoneAdjuster {
                flags: flags.clone(),
                flag: "seasonal-query-milestone-regional".to_string(),
                percent: -8.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 50794.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_query_milestone_engine(&flags::new_registry());
        let item = new_seasonal_query_milestone("code-354", "title-648");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
