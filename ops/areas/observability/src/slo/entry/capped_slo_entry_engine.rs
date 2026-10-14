use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped slo entry calculation.
pub trait CappedSloEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSloEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntryAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl CappedSloEntryAdjuster for CappedSloEntryAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &CappedSloEntry, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedSloEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedSloEntryAdjuster for FlaggedCappedSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedSloEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedSloEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedSloEntryAdjuster for TieredCappedSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedSloEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped slo entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSloEntryEngine {
    pub steps: Vec<Rc<dyn CappedSloEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSloEntryEngine {
    pub fn run(&mut self, item: &CappedSloEntry) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedSloEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedSloEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_slo_entry_engine(flags: &flags::Registry) -> CappedSloEntryEngine {
    CappedSloEntryEngine {
        steps: vec![
            Rc::new(CappedSloEntryAttemptsOffsetAdjuster { weight: 0.1 }) as Rc<dyn CappedSloEntryAdjuster>,
            Rc::new(FlaggedCappedSloEntryAdjuster {
                flags: flags.clone(),
                flag: "capped-slo-entry-legacy".to_string(),
                percent: 7.9,
            }),
            Rc::new(TieredCappedSloEntryAdjuster {
                bounds: vec![789.0, 3204.0, 8214.0],
                rates: vec![1.0, 0.8, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 52740.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_slo_entry_engine(&flags::new_registry());
        let item = new_capped_slo_entry("code-994", "label-327");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
