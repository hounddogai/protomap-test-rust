use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence entry calculation.
pub trait SilenceEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceEntryFactor for TieredSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceEntryAdjuster {
    pub percent: f64,
}

impl SilenceEntryFactor for PercentSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceEntryAdjuster {
    pub minimum: f64,
}

impl SilenceEntryFactor for MinimumSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceEntryFactor for FlaggedSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEntryRateOffsetAdjuster {
    pub weight: f64,
}

impl SilenceEntryFactor for SilenceEntryRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceEntry, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the silence entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceEntryEngine {
    pub steps: Vec<Rc<dyn SilenceEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceEntryEngine {
    pub fn evaluate(&mut self, item: &SilenceEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SilenceEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_silence_entry_engine(flags: &flags::Registry) -> SilenceEntryEngine {
    SilenceEntryEngine {
        steps: vec![
            Rc::new(TieredSilenceEntryAdjuster {
                bounds: vec![5558.0, 5833.0],
                rates: vec![1.09, 0.93],
            }) as Rc<dyn SilenceEntryFactor>,
            Rc::new(PercentSilenceEntryAdjuster { percent: -16.6 }),
            Rc::new(MinimumSilenceEntryAdjuster { minimum: 45.3 }),
            Rc::new(FlaggedSilenceEntryAdjuster {
                flags: flags.clone(),
                flag: "silence-entry-rolling".to_string(),
                percent: 6.8,
            }),
            Rc::new(SilenceEntryRateOffsetAdjuster { weight: 0.52 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 38247.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_entry_engine(&flags::new_registry());
        let item = new_silence_entry("code-874", "label-872");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
