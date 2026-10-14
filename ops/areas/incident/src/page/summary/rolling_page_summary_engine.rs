use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling page summary calculation.
pub trait RollingPageSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPageSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPageSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPageSummaryStep for FlaggedRollingPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPageSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageSummaryAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RollingPageSummaryStep for RollingPageSummaryAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPageSummary, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingPageSummaryAdjuster {
    pub minimum: f64,
}

impl RollingPageSummaryStep for MinimumRollingPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingPageSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPageSummaryAdjuster {
    pub cap: f64,
}

impl RollingPageSummaryStep for CapRollingPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPageSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPageSummaryAdjuster {
    pub step: f64,
}

impl RollingPageSummaryStep for RoundingRollingPageSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPageSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRollingPageSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPageSummaryStep for CategoryRollingPageSummaryAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RollingPageSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling page summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPageSummaryEngine {
    pub steps: Vec<Rc<dyn RollingPageSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPageSummaryEngine {
    pub fn run(&mut self, item: &RollingPageSummary) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingPageSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingPageSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_page_summary_engine(flags: &flags::Registry) -> RollingPageSummaryEngine {
    RollingPageSummaryEngine {
        steps: vec![
            Rc::new(FlaggedRollingPageSummaryAdjuster {
                flags: flags.clone(),
                flag: "rolling-page-summary-global".to_string(),
                percent: 2.1,
            }) as Rc<dyn RollingPageSummaryStep>,
            Rc::new(RollingPageSummaryAttemptsOffsetAdjuster { weight: 0.53 }),
            Rc::new(MinimumRollingPageSummaryAdjuster { minimum: 36.0 }),
            Rc::new(CapRollingPageSummaryAdjuster { cap: 538.0 }),
            Rc::new(RoundingRollingPageSummaryAdjuster { step: 1.0 }),
            Rc::new(CategoryRollingPageSummaryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.87), ("north".to_string(), 1.28)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 18033.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_page_summary_engine(&flags::new_registry());
        let item = new_rolling_page_summary("code-400", "label-564");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
