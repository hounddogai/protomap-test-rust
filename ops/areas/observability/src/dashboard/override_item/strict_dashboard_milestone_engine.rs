use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict dashboard milestone calculation.
pub trait StrictDashboardMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictDashboardMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictDashboardMilestoneAdjuster {
    pub cap: f64,
}

impl StrictDashboardMilestoneStep for CapStrictDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictDashboardMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictDashboardMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictDashboardMilestoneStep for NameStrictDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictDashboardMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictDashboardMilestoneAdjuster {
    pub step: f64,
}

impl StrictDashboardMilestoneStep for RoundingStrictDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictDashboardMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictDashboardMilestoneAdjuster {
    pub minimum: f64,
}

impl StrictDashboardMilestoneStep for MinimumStrictDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictDashboardMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardMilestoneEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl StrictDashboardMilestoneStep for StrictDashboardMilestoneEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &StrictDashboardMilestone, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the strict dashboard milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictDashboardMilestoneEngine {
    pub steps: Vec<Rc<dyn StrictDashboardMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictDashboardMilestoneEngine {
    pub fn compute(&mut self, item: &StrictDashboardMilestone) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictDashboardMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictDashboardMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_dashboard_milestone_engine() -> StrictDashboardMilestoneEngine {
    StrictDashboardMilestoneEngine {
        steps: vec![
            Rc::new(CapStrictDashboardMilestoneAdjuster { cap: 20551.0 }) as Rc<dyn StrictDashboardMilestoneStep>,
            Rc::new(NameStrictDashboardMilestoneAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.25)]),
            }),
            Rc::new(RoundingStrictDashboardMilestoneAdjuster { step: 1.0 }),
            Rc::new(MinimumStrictDashboardMilestoneAdjuster { minimum: 9.3 }),
            Rc::new(StrictDashboardMilestoneEndsAtOffsetAdjuster { weight: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 44337.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_dashboard_milestone_engine();
        let item = new_strict_dashboard_milestone("code-858", "name-607");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
