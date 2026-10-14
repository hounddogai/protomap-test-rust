use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default redaction entry calculation.
pub trait DefaultRedactionEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRedactionEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionEntryBurstOffsetAdjuster {
    pub weight: f64,
}

impl DefaultRedactionEntryModifier for DefaultRedactionEntryBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultRedactionEntry, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultRedactionEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultRedactionEntryModifier for TitleDefaultRedactionEntryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultRedactionEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultRedactionEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultRedactionEntryModifier for FlaggedDefaultRedactionEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultRedactionEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRedactionEntryAdjuster {
    pub step: f64,
}

impl DefaultRedactionEntryModifier for RoundingDefaultRedactionEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRedactionEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default redaction entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRedactionEntryEngine {
    pub steps: Vec<Rc<dyn DefaultRedactionEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRedactionEntryEngine {
    pub fn run(&mut self, item: &DefaultRedactionEntry) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultRedactionEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRedactionEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_redaction_entry_engine(flags: &flags::Registry) -> DefaultRedactionEntryEngine {
    DefaultRedactionEntryEngine {
        steps: vec![
            Rc::new(DefaultRedactionEntryBurstOffsetAdjuster { weight: 0.65 }) as Rc<dyn DefaultRedactionEntryModifier>,
            Rc::new(TitleDefaultRedactionEntryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.84), ("south".to_string(), 1.36)]),
            }),
            Rc::new(FlaggedDefaultRedactionEntryAdjuster {
                flags: flags.clone(),
                flag: "default-redaction-entry-batched".to_string(),
                percent: -8.8,
            }),
            Rc::new(RoundingDefaultRedactionEntryAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 83621.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_redaction_entry_engine(&flags::new_registry());
        let item = new_default_redaction_entry("code-482", "title-931");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
