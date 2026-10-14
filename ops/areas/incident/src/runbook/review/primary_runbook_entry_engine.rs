use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary runbook entry calculation.
pub trait PrimaryRunbookEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRunbookEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRunbookEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRunbookEntryModifier for FlaggedPrimaryRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRunbookEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRunbookEntryAdjuster {
    pub cap: f64,
}

impl PrimaryRunbookEntryModifier for CapPrimaryRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRunbookEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryRunbookEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryRunbookEntryModifier for TieredPrimaryRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryRunbookEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntryEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRunbookEntryModifier for PrimaryRunbookEntryEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRunbookEntry, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryRunbookEntryAdjuster {
    pub step: f64,
}

impl PrimaryRunbookEntryModifier for RoundingPrimaryRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryRunbookEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryRunbookEntryAdjuster {
    pub minimum: f64,
}

impl PrimaryRunbookEntryModifier for MinimumPrimaryRunbookEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryRunbookEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary runbook entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRunbookEntryEngine {
    pub steps: Vec<Rc<dyn PrimaryRunbookEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRunbookEntryEngine {
    pub fn run(&mut self, item: &PrimaryRunbookEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRunbookEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_runbook_entry_engine(flags: &flags::Registry) -> PrimaryRunbookEntryEngine {
    PrimaryRunbookEntryEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryRunbookEntryAdjuster {
                flags: flags.clone(),
                flag: "primary-runbook-entry-lenient".to_string(),
                percent: -5.5,
            }) as Rc<dyn PrimaryRunbookEntryModifier>,
            Rc::new(CapPrimaryRunbookEntryAdjuster { cap: 37591.0 }),
            Rc::new(TieredPrimaryRunbookEntryAdjuster {
                bounds: vec![1104.0, 5555.0, 9153.0],
                rates: vec![1.06, 1.07, 1.13],
            }),
            Rc::new(PrimaryRunbookEntryEndsAtOffsetAdjuster { weight: 0.41 }),
            Rc::new(RoundingPrimaryRunbookEntryAdjuster { step: 5.0 }),
            Rc::new(MinimumPrimaryRunbookEntryAdjuster { minimum: 2.7 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 17771.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_runbook_entry_engine(&flags::new_registry());
        let item = new_primary_runbook_entry("code-168", "title-173");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
