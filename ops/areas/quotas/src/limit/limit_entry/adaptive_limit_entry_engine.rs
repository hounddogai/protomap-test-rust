use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive limit entry calculation.
pub trait AdaptiveLimitEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveLimitEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAdaptiveLimitEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveLimitEntryFactor for LocaleAdaptiveLimitEntryAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AdaptiveLimitEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveLimitEntryAdjuster {
    pub minimum: f64,
}

impl AdaptiveLimitEntryFactor for MinimumAdaptiveLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveLimitEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveLimitEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveLimitEntryFactor for TieredAdaptiveLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveLimitEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveLimitEntryAdjuster {
    pub cap: f64,
}

impl AdaptiveLimitEntryFactor for CapAdaptiveLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveLimitEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveLimitEntryAdjuster {
    pub step: f64,
}

impl AdaptiveLimitEntryFactor for RoundingAdaptiveLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveLimitEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveLimitEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveLimitEntryFactor for FlaggedAdaptiveLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveLimitEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive limit entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveLimitEntryEngine {
    pub steps: Vec<Rc<dyn AdaptiveLimitEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveLimitEntryEngine {
    pub fn run(&mut self, item: &AdaptiveLimitEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveLimitEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_limit_entry_engine(flags: &flags::Registry) -> AdaptiveLimitEntryEngine {
    AdaptiveLimitEntryEngine {
        steps: vec![
            Rc::new(LocaleAdaptiveLimitEntryAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.3),
                    ("central".to_string(), 1.08),
                    ("retail".to_string(), 1.09),
                ]),
            }) as Rc<dyn AdaptiveLimitEntryFactor>,
            Rc::new(MinimumAdaptiveLimitEntryAdjuster { minimum: 17.3 }),
            Rc::new(TieredAdaptiveLimitEntryAdjuster {
                bounds: vec![4910.0, 6775.0, 8007.0],
                rates: vec![0.94, 0.99, 1.06],
            }),
            Rc::new(CapAdaptiveLimitEntryAdjuster { cap: 4933.0 }),
            Rc::new(RoundingAdaptiveLimitEntryAdjuster { step: 1.0 }),
            Rc::new(FlaggedAdaptiveLimitEntryAdjuster {
                flags: flags.clone(),
                flag: "adaptive-limit-entry-manual".to_string(),
                percent: -4.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 22908.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_limit_entry_engine(&flags::new_registry());
        let item = new_adaptive_limit_entry("code-609", "name-72");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
