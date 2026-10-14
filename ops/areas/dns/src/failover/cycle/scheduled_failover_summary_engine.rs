use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled failover summary calculation.
pub trait ScheduledFailoverSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledFailoverSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSummaryPriorityOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledFailoverSummaryFactor for ScheduledFailoverSummaryPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledFailoverSummary, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledFailoverSummaryAdjuster {
    pub cap: f64,
}

impl ScheduledFailoverSummaryFactor for CapScheduledFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledFailoverSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledFailoverSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledFailoverSummaryFactor for FlaggedScheduledFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledFailoverSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledFailoverSummaryAdjuster {
    pub step: f64,
}

impl ScheduledFailoverSummaryFactor for RoundingScheduledFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledFailoverSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled failover summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledFailoverSummaryEngine {
    pub steps: Vec<Rc<dyn ScheduledFailoverSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledFailoverSummaryEngine {
    pub fn evaluate(&mut self, item: &ScheduledFailoverSummary) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledFailoverSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_failover_summary_engine(flags: &flags::Registry) -> ScheduledFailoverSummaryEngine {
    ScheduledFailoverSummaryEngine {
        steps: vec![
            Rc::new(ScheduledFailoverSummaryPriorityOffsetAdjuster { weight: 1.61 })
                as Rc<dyn ScheduledFailoverSummaryFactor>,
            Rc::new(CapScheduledFailoverSummaryAdjuster { cap: 35394.0 }),
            Rc::new(FlaggedScheduledFailoverSummaryAdjuster {
                flags: flags.clone(),
                flag: "scheduled-failover-summary-pending".to_string(),
                percent: -9.8,
            }),
            Rc::new(RoundingScheduledFailoverSummaryAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 71864.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_failover_summary_engine(&flags::new_registry());
        let item = new_scheduled_failover_summary("code-608", "name-453");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
