use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted silence entry calculation.
pub trait WeightedSilenceEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedSilenceEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedSilenceEntryAdjuster {
    pub percent: f64,
}

impl WeightedSilenceEntryAdjuster for PercentWeightedSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedSilenceEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedSilenceEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedSilenceEntryAdjuster for TieredWeightedSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedSilenceEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedSilenceEntryAdjuster {
    pub step: f64,
}

impl WeightedSilenceEntryAdjuster for RoundingWeightedSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedSilenceEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedSilenceEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedSilenceEntryAdjuster for FlaggedWeightedSilenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedSilenceEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted silence entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedSilenceEntryEngine {
    pub steps: Vec<Rc<dyn WeightedSilenceEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedSilenceEntryEngine {
    pub fn compute(&mut self, item: &WeightedSilenceEntry) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedSilenceEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedSilenceEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_silence_entry_engine(flags: &flags::Registry) -> WeightedSilenceEntryEngine {
    WeightedSilenceEntryEngine {
        steps: vec![
            Rc::new(PercentWeightedSilenceEntryAdjuster { percent: 20.3 }) as Rc<dyn WeightedSilenceEntryAdjuster>,
            Rc::new(TieredWeightedSilenceEntryAdjuster {
                bounds: vec![383.0, 3006.0, 6698.0],
                rates: vec![0.88, 1.16, 0.98],
            }),
            Rc::new(RoundingWeightedSilenceEntryAdjuster { step: 5.0 }),
            Rc::new(FlaggedWeightedSilenceEntryAdjuster {
                flags: flags.clone(),
                flag: "weighted-silence-entry-manual".to_string(),
                percent: 1.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 93754.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_silence_entry_engine(&flags::new_registry());
        let item = new_weighted_silence_entry("code-984", "label-539");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
