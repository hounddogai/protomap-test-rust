use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled dashboard rule calculation.
pub trait ScheduledDashboardRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDashboardRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDashboardRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDashboardRuleModifier for FlaggedScheduledDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDashboardRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledDashboardRuleAdjuster {
    pub percent: f64,
}

impl ScheduledDashboardRuleModifier for PercentScheduledDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledDashboardRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDashboardRuleAdjuster {
    pub minimum: f64,
}

impl ScheduledDashboardRuleModifier for MinimumScheduledDashboardRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDashboardRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled dashboard rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDashboardRuleEngine {
    pub steps: Vec<Rc<dyn ScheduledDashboardRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDashboardRuleEngine {
    pub fn compute(&mut self, item: &ScheduledDashboardRule) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDashboardRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_dashboard_rule_engine(flags: &flags::Registry) -> ScheduledDashboardRuleEngine {
    ScheduledDashboardRuleEngine {
        steps: vec![
            Rc::new(FlaggedScheduledDashboardRuleAdjuster {
                flags: flags.clone(),
                flag: "scheduled-dashboard-rule-rolling".to_string(),
                percent: -0.2,
            }) as Rc<dyn ScheduledDashboardRuleModifier>,
            Rc::new(PercentScheduledDashboardRuleAdjuster { percent: 23.1 }),
            Rc::new(MinimumScheduledDashboardRuleAdjuster { minimum: 8.7 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 98616.2,
    }
}
