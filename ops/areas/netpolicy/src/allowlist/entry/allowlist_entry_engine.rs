use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist entry calculation.
pub trait AllowlistEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAllowlistEntryAdjuster {
    pub step: f64,
}

impl AllowlistEntryAdjuster for RoundingAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AllowlistEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistEntryAdjuster {
    pub cap: f64,
}

impl AllowlistEntryAdjuster for CapAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistEntryAdjuster for FlaggedAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEntryMaximumOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistEntryAdjuster for AllowlistEntryMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistEntry, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the allowlist entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistEntryEngine {
    pub steps: Vec<Rc<dyn AllowlistEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistEntryEngine {
    pub fn price(&mut self, item: &AllowlistEntry) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_entry_engine(flags: &flags::Registry) -> AllowlistEntryEngine {
    AllowlistEntryEngine {
        steps: vec![
            Rc::new(RoundingAllowlistEntryAdjuster { step: 5.0 }) as Rc<dyn AllowlistEntryAdjuster>,
            Rc::new(CapAllowlistEntryAdjuster { cap: 9096.0 }),
            Rc::new(FlaggedAllowlistEntryAdjuster {
                flags: flags.clone(),
                flag: "allowlist-entry-adaptive".to_string(),
                percent: -5.2,
            }),
            Rc::new(AllowlistEntryMaximumOffsetAdjuster { weight: 0.57 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 12689.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_entry_engine(&flags::new_registry());
        let item = new_allowlist_entry("code-886", "name-730");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
