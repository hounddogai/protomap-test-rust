use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction entry calculation.
pub trait RedactionEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionEntryFactor for FlaggedRedactionEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionEntryAdjuster {
    pub step: f64,
}

impl RedactionEntryFactor for RoundingRedactionEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionEntryAdjuster {
    pub minimum: f64,
}

impl RedactionEntryFactor for MinimumRedactionEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the redaction entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionEntryEngine {
    pub steps: Vec<Rc<dyn RedactionEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionEntryEngine {
    pub fn run(&mut self, item: &RedactionEntry) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionEntryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_redaction_entry_engine(flags: &flags::Registry) -> RedactionEntryEngine {
    RedactionEntryEngine {
        steps: vec![
            Rc::new(FlaggedRedactionEntryAdjuster {
                flags: flags.clone(),
                flag: "redaction-entry-primary".to_string(),
                percent: 6.5,
            }) as Rc<dyn RedactionEntryFactor>,
            Rc::new(RoundingRedactionEntryAdjuster { step: 0.25 }),
            Rc::new(MinimumRedactionEntryAdjuster { minimum: 21.9 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 41792.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_entry_engine(&flags::new_registry());
        let item = new_redaction_entry("code-793", "title-356");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
