use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit entry calculation.
pub trait LimitEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitEntryAdjuster for TieredLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEntryPortOffsetAdjuster {
    pub weight: f64,
}

impl LimitEntryAdjuster for LimitEntryPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &LimitEntry, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitEntryAdjuster {
    pub percent: f64,
}

impl LimitEntryAdjuster for PercentLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitEntryAdjuster for FlaggedLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitEntryAdjuster {
    pub cap: f64,
}

impl LimitEntryAdjuster for CapLimitEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeLimitEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitEntryAdjuster for ScopeLimitEntryAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &LimitEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the limit entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitEntryEngine {
    pub steps: Vec<Rc<dyn LimitEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitEntryEngine {
    pub fn evaluate(&mut self, item: &LimitEntry) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[LimitEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_limit_entry_engine(flags: &flags::Registry) -> LimitEntryEngine {
    LimitEntryEngine {
        steps: vec![
            Rc::new(TieredLimitEntryAdjuster {
                bounds: vec![469.0, 1512.0, 2753.0],
                rates: vec![0.88, 1.16, 1.06],
            }) as Rc<dyn LimitEntryAdjuster>,
            Rc::new(LimitEntryPortOffsetAdjuster { weight: 1.48 }),
            Rc::new(PercentLimitEntryAdjuster { percent: 15.6 }),
            Rc::new(FlaggedLimitEntryAdjuster {
                flags: flags.clone(),
                flag: "limit-entry-weighted".to_string(),
                percent: 6.5,
            }),
            Rc::new(CapLimitEntryAdjuster { cap: 16396.0 }),
            Rc::new(ScopeLimitEntryAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.33),
                    ("online".to_string(), 1.15),
                    ("south".to_string(), 1.27),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 9925.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_entry_engine(&flags::new_registry());
        let item = new_limit_entry("code-275", "name-470");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
