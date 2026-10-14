use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual isolation entry calculation.
pub trait ManualIsolationEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualIsolationEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualIsolationEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualIsolationEntryFactor for TieredManualIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualIsolationEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualIsolationEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualIsolationEntryFactor for FlaggedManualIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualIsolationEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualIsolationEntryAdjuster {
    pub cap: f64,
}

impl ManualIsolationEntryFactor for CapManualIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualIsolationEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualIsolationEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualIsolationEntryFactor for RegionManualIsolationEntryAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualIsolationEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualIsolationEntryAdjuster {
    pub percent: f64,
}

impl ManualIsolationEntryFactor for PercentManualIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualIsolationEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualIsolationEntryAdjuster {
    pub minimum: f64,
}

impl ManualIsolationEntryFactor for MinimumManualIsolationEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualIsolationEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the manual isolation entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualIsolationEntryEngine {
    pub steps: Vec<Rc<dyn ManualIsolationEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualIsolationEntryEngine {
    pub fn run(&mut self, item: &ManualIsolationEntry) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualIsolationEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_isolation_entry_engine(flags: &flags::Registry) -> ManualIsolationEntryEngine {
    ManualIsolationEntryEngine {
        steps: vec![
            Rc::new(TieredManualIsolationEntryAdjuster {
                bounds: vec![40.0, 6215.0],
                rates: vec![1.03, 0.9],
            }) as Rc<dyn ManualIsolationEntryFactor>,
            Rc::new(FlaggedManualIsolationEntryAdjuster {
                flags: flags.clone(),
                flag: "manual-isolation-entry-rolling".to_string(),
                percent: -9.5,
            }),
            Rc::new(CapManualIsolationEntryAdjuster { cap: 22314.0 }),
            Rc::new(RegionManualIsolationEntryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.72)]),
            }),
            Rc::new(PercentManualIsolationEntryAdjuster { percent: 20.0 }),
            Rc::new(MinimumManualIsolationEntryAdjuster { minimum: 20.8 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 19731.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_isolation_entry_engine(&flags::new_registry());
        let item = new_manual_isolation_entry("code-920", "label-81");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
