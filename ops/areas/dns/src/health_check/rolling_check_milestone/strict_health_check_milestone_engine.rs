use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict health check milestone calculation.
pub trait StrictHealthCheckMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictHealthCheckMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictHealthCheckMilestoneAdjuster {
    pub minimum: f64,
}

impl StrictHealthCheckMilestoneStep for MinimumStrictHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictHealthCheckMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestoneMarginOffsetAdjuster {
    pub weight: f64,
}

impl StrictHealthCheckMilestoneStep for StrictHealthCheckMilestoneMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &StrictHealthCheckMilestone, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictHealthCheckMilestoneAdjuster {
    pub cap: f64,
}

impl StrictHealthCheckMilestoneStep for CapStrictHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictHealthCheckMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceStrictHealthCheckMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictHealthCheckMilestoneStep for NamespaceStrictHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &StrictHealthCheckMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictHealthCheckMilestoneAdjuster {
    pub percent: f64,
}

impl StrictHealthCheckMilestoneStep for PercentStrictHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictHealthCheckMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictHealthCheckMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictHealthCheckMilestoneStep for FlaggedStrictHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictHealthCheckMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict health check milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictHealthCheckMilestoneEngine {
    pub steps: Vec<Rc<dyn StrictHealthCheckMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictHealthCheckMilestoneEngine {
    pub fn evaluate(&mut self, item: &StrictHealthCheckMilestone) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictHealthCheckMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictHealthCheckMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_health_check_milestone_engine(flags: &flags::Registry) -> StrictHealthCheckMilestoneEngine {
    StrictHealthCheckMilestoneEngine {
        steps: vec![
            Rc::new(MinimumStrictHealthCheckMilestoneAdjuster { minimum: 6.5 })
                as Rc<dyn StrictHealthCheckMilestoneStep>,
            Rc::new(StrictHealthCheckMilestoneMarginOffsetAdjuster { weight: 1.34 }),
            Rc::new(CapStrictHealthCheckMilestoneAdjuster { cap: 16517.0 }),
            Rc::new(NamespaceStrictHealthCheckMilestoneAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.01), ("south".to_string(), 0.76)]),
            }),
            Rc::new(PercentStrictHealthCheckMilestoneAdjuster { percent: 8.0 }),
            Rc::new(FlaggedStrictHealthCheckMilestoneAdjuster {
                flags: flags.clone(),
                flag: "strict-health-check-milestone-capped".to_string(),
                percent: 8.6,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 22392.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_health_check_milestone_engine(&flags::new_registry());
        let item = new_strict_health_check_milestone("code-326", "title-802");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
