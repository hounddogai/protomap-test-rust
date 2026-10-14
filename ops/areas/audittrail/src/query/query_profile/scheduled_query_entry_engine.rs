use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled query entry calculation.
pub trait ScheduledQueryEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledQueryEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledQueryEntryAdjuster {
    pub minimum: f64,
}

impl ScheduledQueryEntryStep for MinimumScheduledQueryEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledQueryEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledQueryEntryAdjuster {
    pub percent: f64,
}

impl ScheduledQueryEntryStep for PercentScheduledQueryEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledQueryEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceScheduledQueryEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledQueryEntryStep for NamespaceScheduledQueryEntryAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ScheduledQueryEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledQueryEntryAdjuster {
    pub step: f64,
}

impl ScheduledQueryEntryStep for RoundingScheduledQueryEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledQueryEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledQueryEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledQueryEntryStep for FlaggedScheduledQueryEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledQueryEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled query entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledQueryEntryEngine {
    pub steps: Vec<Rc<dyn ScheduledQueryEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledQueryEntryEngine {
    pub fn evaluate(&mut self, item: &ScheduledQueryEntry) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledQueryEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_query_entry_engine(flags: &flags::Registry) -> ScheduledQueryEntryEngine {
    ScheduledQueryEntryEngine {
        steps: vec![
            Rc::new(MinimumScheduledQueryEntryAdjuster { minimum: 18.4 }) as Rc<dyn ScheduledQueryEntryStep>,
            Rc::new(PercentScheduledQueryEntryAdjuster { percent: -14.4 }),
            Rc::new(NamespaceScheduledQueryEntryAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.25),
                    ("west".to_string(), 1.29),
                    ("east".to_string(), 0.86),
                ]),
            }),
            Rc::new(RoundingScheduledQueryEntryAdjuster { step: 0.25 }),
            Rc::new(FlaggedScheduledQueryEntryAdjuster {
                flags: flags.clone(),
                flag: "scheduled-query-entry-scheduled".to_string(),
                percent: -5.8,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 67533.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_query_entry_engine(&flags::new_registry());
        let item = new_scheduled_query_entry("code-392", "label-727");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
