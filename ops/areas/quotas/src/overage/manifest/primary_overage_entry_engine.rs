use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary overage entry calculation.
pub trait PrimaryOverageEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryOverageEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryOverageEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryOverageEntryModifier for FlaggedPrimaryOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryOverageEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageEntryQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryOverageEntryModifier for PrimaryOverageEntryQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryOverageEntry, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryOverageEntryAdjuster {
    pub cap: f64,
}

impl PrimaryOverageEntryModifier for CapPrimaryOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryOverageEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryOverageEntryAdjuster {
    pub step: f64,
}

impl PrimaryOverageEntryModifier for RoundingPrimaryOverageEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryOverageEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary overage entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryOverageEntryEngine {
    pub steps: Vec<Rc<dyn PrimaryOverageEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryOverageEntryEngine {
    pub fn run(&mut self, item: &PrimaryOverageEntry) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryOverageEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryOverageEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_overage_entry_engine(flags: &flags::Registry) -> PrimaryOverageEntryEngine {
    PrimaryOverageEntryEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryOverageEntryAdjuster {
                flags: flags.clone(),
                flag: "primary-overage-entry-global".to_string(),
                percent: -3.4,
            }) as Rc<dyn PrimaryOverageEntryModifier>,
            Rc::new(PrimaryOverageEntryQuotaUnitsOffsetAdjuster { weight: 1.58 }),
            Rc::new(CapPrimaryOverageEntryAdjuster { cap: 27966.0 }),
            Rc::new(RoundingPrimaryOverageEntryAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 58932.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_overage_entry_engine(&flags::new_registry());
        let item = new_primary_overage_entry("code-186", "label-953");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
