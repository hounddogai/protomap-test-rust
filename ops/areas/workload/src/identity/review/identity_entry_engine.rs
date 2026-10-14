use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity entry calculation.
pub trait IdentityEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityEntryStep for TieredIdentityEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityEntryStep for FlaggedIdentityEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityEntryAdjuster {
    pub cap: f64,
}

impl IdentityEntryStep for CapIdentityEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityEntryAdjuster {
    pub step: f64,
}

impl IdentityEntryStep for RoundingIdentityEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityEntryAdjuster {
    pub minimum: f64,
}

impl IdentityEntryStep for MinimumIdentityEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntryVersionOffsetAdjuster {
    pub weight: f64,
}

impl IdentityEntryStep for IdentityEntryVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityEntry, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the identity entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityEntryEngine {
    pub steps: Vec<Rc<dyn IdentityEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityEntryEngine {
    pub fn run(&mut self, item: &IdentityEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_identity_entry_engine(flags: &flags::Registry) -> IdentityEntryEngine {
    IdentityEntryEngine {
        steps: vec![
            Rc::new(TieredIdentityEntryAdjuster {
                bounds: vec![431.0, 8004.0],
                rates: vec![0.98, 0.94],
            }) as Rc<dyn IdentityEntryStep>,
            Rc::new(FlaggedIdentityEntryAdjuster {
                flags: flags.clone(),
                flag: "identity-entry-manual".to_string(),
                percent: -1.2,
            }),
            Rc::new(CapIdentityEntryAdjuster { cap: 15817.0 }),
            Rc::new(RoundingIdentityEntryAdjuster { step: 0.5 }),
            Rc::new(MinimumIdentityEntryAdjuster { minimum: 45.6 }),
            Rc::new(IdentityEntryVersionOffsetAdjuster { weight: 1.64 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 74997.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_entry_engine(&flags::new_registry());
        let item = new_identity_entry("code-318", "title-962");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
