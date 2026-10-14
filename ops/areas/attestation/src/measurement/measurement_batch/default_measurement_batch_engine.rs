use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default measurement batch calculation.
pub trait DefaultMeasurementBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultMeasurementBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultMeasurementBatchAdjuster {
    pub cap: f64,
}

impl DefaultMeasurementBatchFactor for CapDefaultMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultMeasurementBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultMeasurementBatchAdjuster {
    pub minimum: f64,
}

impl DefaultMeasurementBatchFactor for MinimumDefaultMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultMeasurementBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultMeasurementBatchAdjuster {
    pub step: f64,
}

impl DefaultMeasurementBatchFactor for RoundingDefaultMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultMeasurementBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultMeasurementBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultMeasurementBatchFactor for TieredDefaultMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultMeasurementBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultMeasurementBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultMeasurementBatchFactor for FlaggedDefaultMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultMeasurementBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDefaultMeasurementBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultMeasurementBatchFactor for TitleDefaultMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DefaultMeasurementBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default measurement batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultMeasurementBatchEngine {
    pub steps: Vec<Rc<dyn DefaultMeasurementBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultMeasurementBatchEngine {
    pub fn compute(&mut self, item: &DefaultMeasurementBatch) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[DefaultMeasurementBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_measurement_batch_engine(flags: &flags::Registry) -> DefaultMeasurementBatchEngine {
    DefaultMeasurementBatchEngine {
        steps: vec![
            Rc::new(CapDefaultMeasurementBatchAdjuster { cap: 4079.0 }) as Rc<dyn DefaultMeasurementBatchFactor>,
            Rc::new(MinimumDefaultMeasurementBatchAdjuster { minimum: 45.5 }),
            Rc::new(RoundingDefaultMeasurementBatchAdjuster { step: 1.0 }),
            Rc::new(TieredDefaultMeasurementBatchAdjuster {
                bounds: vec![1962.0, 7326.0, 9424.0],
                rates: vec![0.85, 1.02, 0.99],
            }),
            Rc::new(FlaggedDefaultMeasurementBatchAdjuster {
                flags: flags.clone(),
                flag: "default-measurement-batch-adaptive".to_string(),
                percent: 0.2,
            }),
            Rc::new(TitleDefaultMeasurementBatchAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.3), ("south".to_string(), 0.99)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 45082.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_measurement_batch_engine(&flags::new_registry());
        let item = new_default_measurement_batch("code-41", "title-709");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
