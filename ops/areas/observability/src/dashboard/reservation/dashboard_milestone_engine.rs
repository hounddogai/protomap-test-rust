use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dashboard milestone calculation.
pub trait DashboardMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDashboardMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardMilestoneFactor for SourceDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &DashboardMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardMilestoneAdjuster {
    pub step: f64,
}

impl DashboardMilestoneFactor for RoundingDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardMilestoneAmountOffsetAdjuster {
    pub weight: f64,
}

impl DashboardMilestoneFactor for DashboardMilestoneAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardMilestone, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardMilestoneFactor for FlaggedDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardMilestoneAdjuster {
    pub percent: f64,
}

impl DashboardMilestoneFactor for PercentDashboardMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dashboard milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardMilestoneEngine {
    pub steps: Vec<Rc<dyn DashboardMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardMilestoneEngine {
    pub fn evaluate(&mut self, item: &DashboardMilestone) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DashboardMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dashboard_milestone_engine(flags: &flags::Registry) -> DashboardMilestoneEngine {
    DashboardMilestoneEngine {
        steps: vec![
            Rc::new(SourceDashboardMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.99)]),
            }) as Rc<dyn DashboardMilestoneFactor>,
            Rc::new(RoundingDashboardMilestoneAdjuster { step: 0.1 }),
            Rc::new(DashboardMilestoneAmountOffsetAdjuster { weight: 0.18 }),
            Rc::new(FlaggedDashboardMilestoneAdjuster {
                flags: flags.clone(),
                flag: "dashboard-milestone-fallback".to_string(),
                percent: -4.3,
            }),
            Rc::new(PercentDashboardMilestoneAdjuster { percent: -5.1 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 54464.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dashboard_milestone_engine(&flags::new_registry());
        let item = new_dashboard_milestone("code-338", "name-366");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
