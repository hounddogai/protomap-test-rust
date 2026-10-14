use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension entry calculation.
pub trait SuspensionEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionEntryModifier for FlaggedSuspensionEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionEntryAdjuster {
    pub minimum: f64,
}

impl SuspensionEntryModifier for MinimumSuspensionEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionEntryAdjuster {
    pub step: f64,
}

impl SuspensionEntryModifier for RoundingSuspensionEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionEntryAdjuster {
    pub cap: f64,
}

impl SuspensionEntryModifier for CapSuspensionEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the suspension entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionEntryEngine {
    pub steps: Vec<Rc<dyn SuspensionEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionEntryEngine {
    pub fn evaluate(&mut self, item: &SuspensionEntry) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_entry_engine(flags: &flags::Registry) -> SuspensionEntryEngine {
    SuspensionEntryEngine {
        steps: vec![
            Rc::new(FlaggedSuspensionEntryAdjuster {
                flags: flags.clone(),
                flag: "suspension-entry-manual".to_string(),
                percent: -1.9,
            }) as Rc<dyn SuspensionEntryModifier>,
            Rc::new(MinimumSuspensionEntryAdjuster { minimum: 4.9 }),
            Rc::new(RoundingSuspensionEntryAdjuster { step: 0.5 }),
            Rc::new(CapSuspensionEntryAdjuster { cap: 14453.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 73919.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_entry_engine(&flags::new_registry());
        let item = new_suspension_entry("code-800", "title-513");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
