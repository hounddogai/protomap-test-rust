use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant entry calculation.
pub trait GrantEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantEntryAdjuster for TieredGrantEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantEntryTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl GrantEntryAdjuster for GrantEntryTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &GrantEntry, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantEntryAdjuster {
    pub minimum: f64,
}

impl GrantEntryAdjuster for MinimumGrantEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGrantEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantEntryAdjuster for NameGrantEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GrantEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantEntryAdjuster for FlaggedGrantEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the grant entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantEntryEngine {
    pub steps: Vec<Rc<dyn GrantEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantEntryEngine {
    pub fn run(&mut self, item: &GrantEntry) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[GrantEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_grant_entry_engine(flags: &flags::Registry) -> GrantEntryEngine {
    GrantEntryEngine {
        steps: vec![
            Rc::new(TieredGrantEntryAdjuster {
                bounds: vec![3089.0, 3906.0, 6708.0, 8029.0],
                rates: vec![0.95, 0.84, 0.8, 0.84],
            }) as Rc<dyn GrantEntryAdjuster>,
            Rc::new(GrantEntryTtlSecondsOffsetAdjuster { weight: 0.69 }),
            Rc::new(MinimumGrantEntryAdjuster { minimum: 28.1 }),
            Rc::new(NameGrantEntryAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.17),
                    ("north".to_string(), 0.8),
                    ("online".to_string(), 0.88),
                ]),
            }),
            Rc::new(FlaggedGrantEntryAdjuster {
                flags: flags.clone(),
                flag: "grant-entry-fallback".to_string(),
                percent: 4.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 24693.3,
    }
}
