use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled credential window calculation.
pub trait ScheduledCredentialWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledCredentialWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledCredentialWindowAdjuster {
    pub minimum: f64,
}

impl ScheduledCredentialWindowStep for MinimumScheduledCredentialWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledCredentialWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledCredentialWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledCredentialWindowStep for FlaggedScheduledCredentialWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledCredentialWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledCredentialWindowAdjuster {
    pub cap: f64,
}

impl ScheduledCredentialWindowStep for CapScheduledCredentialWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledCredentialWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled credential window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledCredentialWindowEngine {
    pub steps: Vec<Rc<dyn ScheduledCredentialWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledCredentialWindowEngine {
    pub fn price(&mut self, item: &ScheduledCredentialWindow) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledCredentialWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_credential_window_engine(flags: &flags::Registry) -> ScheduledCredentialWindowEngine {
    ScheduledCredentialWindowEngine {
        steps: vec![
            Rc::new(MinimumScheduledCredentialWindowAdjuster { minimum: 23.3 })
                as Rc<dyn ScheduledCredentialWindowStep>,
            Rc::new(FlaggedScheduledCredentialWindowAdjuster {
                flags: flags.clone(),
                flag: "scheduled-credential-window-fallback".to_string(),
                percent: -2.4,
            }),
            Rc::new(CapScheduledCredentialWindowAdjuster { cap: 27856.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 71388.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_credential_window_engine(&flags::new_registry());
        let item = new_scheduled_credential_window("code-188", "label-290");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
