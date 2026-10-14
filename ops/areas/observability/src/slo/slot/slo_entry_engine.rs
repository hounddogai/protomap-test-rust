use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo entry calculation.
pub trait SloEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloEntryAdjuster {
    pub step: f64,
}

impl SloEntryStep for RoundingSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloEntryRateOffsetAdjuster {
    pub weight: f64,
}

impl SloEntryStep for SloEntryRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &SloEntry, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloEntryAdjuster {
    pub minimum: f64,
}

impl SloEntryStep for MinimumSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSloEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloEntryStep for SourceSloEntryAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SloEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloEntryStep for TieredSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloEntryAdjuster {
    pub percent: f64,
}

impl SloEntryStep for PercentSloEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the slo entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloEntryEngine {
    pub steps: Vec<Rc<dyn SloEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloEntryEngine {
    pub fn run(&mut self, item: &SloEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SloEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_slo_entry_engine() -> SloEntryEngine {
    SloEntryEngine {
        steps: vec![
            Rc::new(RoundingSloEntryAdjuster { step: 5.0 }) as Rc<dyn SloEntryStep>,
            Rc::new(SloEntryRateOffsetAdjuster { weight: 1.02 }),
            Rc::new(MinimumSloEntryAdjuster { minimum: 37.0 }),
            Rc::new(SourceSloEntryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.87), ("north".to_string(), 1.05)]),
            }),
            Rc::new(TieredSloEntryAdjuster {
                bounds: vec![2965.0, 6441.0],
                rates: vec![0.95, 1.08],
            }),
            Rc::new(PercentSloEntryAdjuster { percent: -23.4 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 34013.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_entry_engine();
        let item = new_slo_entry("code-456", "name-303");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
