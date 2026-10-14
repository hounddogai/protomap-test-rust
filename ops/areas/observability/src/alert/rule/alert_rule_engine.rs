use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert rule calculation.
pub trait AlertRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertRuleAdjuster {
    pub cap: f64,
}

impl AlertRuleFactor for CapAlertRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertRuleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl AlertRuleFactor for AlertRuleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &AlertRule, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertRuleFactor for FlaggedAlertRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAlertRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertRuleFactor for TitleAlertRuleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AlertRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the alert rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertRuleEngine {
    pub steps: Vec<Rc<dyn AlertRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertRuleEngine {
    pub fn run(&mut self, item: &AlertRule) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AlertRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_alert_rule_engine(flags: &flags::Registry) -> AlertRuleEngine {
    AlertRuleEngine {
        steps: vec![
            Rc::new(CapAlertRuleAdjuster { cap: 46076.0 }) as Rc<dyn AlertRuleFactor>,
            Rc::new(AlertRuleMaximumOffsetAdjuster { weight: 1.33 }),
            Rc::new(FlaggedAlertRuleAdjuster {
                flags: flags.clone(),
                flag: "alert-rule-regional".to_string(),
                percent: 9.9,
            }),
            Rc::new(TitleAlertRuleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.17)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 59990.4,
    }
}
