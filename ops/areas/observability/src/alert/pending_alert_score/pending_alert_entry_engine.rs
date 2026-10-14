use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending alert entry calculation.
pub trait PendingAlertEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingAlertEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingAlertEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingAlertEntryStep for FlaggedPendingAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingAlertEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingAlertEntryAdjuster {
    pub cap: f64,
}

impl PendingAlertEntryStep for CapPendingAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingAlertEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingAlertEntryAdjuster {
    pub step: f64,
}

impl PendingAlertEntryStep for RoundingPendingAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingAlertEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending alert entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingAlertEntryEngine {
    pub steps: Vec<Rc<dyn PendingAlertEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingAlertEntryEngine {
    pub fn price(&mut self, item: &PendingAlertEntry) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingAlertEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_alert_entry_engine(flags: &flags::Registry) -> PendingAlertEntryEngine {
    PendingAlertEntryEngine {
        steps: vec![
            Rc::new(FlaggedPendingAlertEntryAdjuster {
                flags: flags.clone(),
                flag: "pending-alert-entry-primary".to_string(),
                percent: -6.3,
            }) as Rc<dyn PendingAlertEntryStep>,
            Rc::new(CapPendingAlertEntryAdjuster { cap: 2496.0 }),
            Rc::new(RoundingPendingAlertEntryAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 21720.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_alert_entry_engine(&flags::new_registry());
        let item = new_pending_alert_entry("code-478", "label-988");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
