use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default freeze entry calculation.
pub trait DefaultFreezeEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultFreezeEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultFreezeEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultFreezeEntryFactor for FlaggedDefaultFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultFreezeEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultFreezeEntryAdjuster {
    pub cap: f64,
}

impl DefaultFreezeEntryFactor for CapDefaultFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultFreezeEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryDefaultFreezeEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultFreezeEntryFactor for CategoryDefaultFreezeEntryAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &DefaultFreezeEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultFreezeEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultFreezeEntryFactor for TieredDefaultFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultFreezeEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeEntryDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl DefaultFreezeEntryFactor for DefaultFreezeEntryDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultFreezeEntry, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultFreezeEntryAdjuster {
    pub step: f64,
}

impl DefaultFreezeEntryFactor for RoundingDefaultFreezeEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultFreezeEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default freeze entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultFreezeEntryEngine {
    pub steps: Vec<Rc<dyn DefaultFreezeEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultFreezeEntryEngine {
    pub fn compute(&mut self, item: &DefaultFreezeEntry) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultFreezeEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_freeze_entry_engine(flags: &flags::Registry) -> DefaultFreezeEntryEngine {
    DefaultFreezeEntryEngine {
        steps: vec![
            Rc::new(FlaggedDefaultFreezeEntryAdjuster {
                flags: flags.clone(),
                flag: "default-freeze-entry-seasonal".to_string(),
                percent: -1.2,
            }) as Rc<dyn DefaultFreezeEntryFactor>,
            Rc::new(CapDefaultFreezeEntryAdjuster { cap: 3199.0 }),
            Rc::new(CategoryDefaultFreezeEntryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.97)]),
            }),
            Rc::new(TieredDefaultFreezeEntryAdjuster {
                bounds: vec![2202.0, 7929.0, 9277.0],
                rates: vec![1.11, 0.83, 1.05],
            }),
            Rc::new(DefaultFreezeEntryDurationMinutesOffsetAdjuster { weight: 1.31 }),
            Rc::new(RoundingDefaultFreezeEntryAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 56608.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_freeze_entry_engine(&flags::new_registry());
        let item = new_default_freeze_entry("code-165", "label-146");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
