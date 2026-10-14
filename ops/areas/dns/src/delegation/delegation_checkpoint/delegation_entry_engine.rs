use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the delegation entry calculation.
pub trait DelegationEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDelegationEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationEntryAdjuster for LabelDelegationEntryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DelegationEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationEntryAdjuster {
    pub minimum: f64,
}

impl DelegationEntryAdjuster for MinimumDelegationEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationEntryAdjuster {
    pub cap: f64,
}

impl DelegationEntryAdjuster for CapDelegationEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationEntryAdjuster for FlaggedDelegationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationEntryAdjuster for TieredDelegationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the delegation entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationEntryEngine {
    pub steps: Vec<Rc<dyn DelegationEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationEntryEngine {
    pub fn price(&mut self, item: &DelegationEntry) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_entry_engine(flags: &flags::Registry) -> DelegationEntryEngine {
    DelegationEntryEngine {
        steps: vec![
            Rc::new(LabelDelegationEntryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.2)]),
            }) as Rc<dyn DelegationEntryAdjuster>,
            Rc::new(MinimumDelegationEntryAdjuster { minimum: 9.0 }),
            Rc::new(CapDelegationEntryAdjuster { cap: 38338.0 }),
            Rc::new(FlaggedDelegationEntryAdjuster {
                flags: flags.clone(),
                flag: "delegation-entry-tiered".to_string(),
                percent: 4.5,
            }),
            Rc::new(TieredDelegationEntryAdjuster {
                bounds: vec![2901.0, 6369.0, 8276.0],
                rates: vec![1.14, 0.99, 0.82],
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 42554.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_entry_engine(&flags::new_registry());
        let item = new_delegation_entry("code-684", "label-926");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
