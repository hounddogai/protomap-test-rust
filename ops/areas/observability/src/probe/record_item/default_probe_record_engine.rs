use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default probe record calculation.
pub trait DefaultProbeRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultProbeRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultProbeRecordAdjuster {
    pub step: f64,
}

impl DefaultProbeRecordStep for RoundingDefaultProbeRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultProbeRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultProbeRecordAdjuster {
    pub minimum: f64,
}

impl DefaultProbeRecordStep for MinimumDefaultProbeRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultProbeRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProbeRecordRetriesOffsetAdjuster {
    pub weight: f64,
}

impl DefaultProbeRecordStep for DefaultProbeRecordRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultProbeRecord, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultProbeRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultProbeRecordStep for TieredDefaultProbeRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultProbeRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryDefaultProbeRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultProbeRecordStep for CountryDefaultProbeRecordAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &DefaultProbeRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default probe record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultProbeRecordEngine {
    pub steps: Vec<Rc<dyn DefaultProbeRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultProbeRecordEngine {
    pub fn evaluate(&mut self, item: &DefaultProbeRecord) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[DefaultProbeRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_probe_record_engine() -> DefaultProbeRecordEngine {
    DefaultProbeRecordEngine {
        steps: vec![
            Rc::new(RoundingDefaultProbeRecordAdjuster { step: 0.25 }) as Rc<dyn DefaultProbeRecordStep>,
            Rc::new(MinimumDefaultProbeRecordAdjuster { minimum: 12.2 }),
            Rc::new(DefaultProbeRecordRetriesOffsetAdjuster { weight: 1.26 }),
            Rc::new(TieredDefaultProbeRecordAdjuster {
                bounds: vec![4026.0, 9429.0],
                rates: vec![0.84, 1.12],
            }),
            Rc::new(CountryDefaultProbeRecordAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.72),
                    ("north".to_string(), 0.93),
                    ("online".to_string(), 1.02),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 75727.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_probe_record_engine();
        let item = new_default_probe_record("code-263", "label-116");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
