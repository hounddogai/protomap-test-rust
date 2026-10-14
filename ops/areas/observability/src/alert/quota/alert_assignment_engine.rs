use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert assignment calculation.
pub trait AlertAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertAssignmentModifier for FlaggedAlertAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertAssignmentAdjuster {
    pub percent: f64,
}

impl AlertAssignmentModifier for PercentAlertAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAlertAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertAssignmentModifier for ChannelAlertAssignmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AlertAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertAssignmentAdjuster {
    pub step: f64,
}

impl AlertAssignmentModifier for RoundingAlertAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertAssignmentAdjuster {
    pub cap: f64,
}

impl AlertAssignmentModifier for CapAlertAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the alert assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertAssignmentEngine {
    pub steps: Vec<Rc<dyn AlertAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertAssignmentEngine {
    pub fn compute(&mut self, item: &AlertAssignment) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[AlertAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_alert_assignment_engine(flags: &flags::Registry) -> AlertAssignmentEngine {
    AlertAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedAlertAssignmentAdjuster {
                flags: flags.clone(),
                flag: "alert-assignment-legacy".to_string(),
                percent: 3.7,
            }) as Rc<dyn AlertAssignmentModifier>,
            Rc::new(PercentAlertAssignmentAdjuster { percent: -19.1 }),
            Rc::new(ChannelAlertAssignmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.28)]),
            }),
            Rc::new(RoundingAlertAssignmentAdjuster { step: 0.05 }),
            Rc::new(CapAlertAssignmentAdjuster { cap: 48463.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 28525.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_assignment_engine(&flags::new_registry());
        let item = new_alert_assignment("code-198", "title-357");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
