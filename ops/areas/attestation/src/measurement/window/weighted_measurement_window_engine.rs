use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted measurement window calculation.
pub trait WeightedMeasurementWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedMeasurementWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedMeasurementWindowAdjuster {
    pub minimum: f64,
}

impl WeightedMeasurementWindowAdjuster for MinimumWeightedMeasurementWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedMeasurementWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedMeasurementWindowAdjuster {
    pub step: f64,
}

impl WeightedMeasurementWindowAdjuster for RoundingWeightedMeasurementWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedMeasurementWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementWindowRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl WeightedMeasurementWindowAdjuster for WeightedMeasurementWindowRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedMeasurementWindow, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedMeasurementWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedMeasurementWindowAdjuster for FlaggedWeightedMeasurementWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedMeasurementWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedMeasurementWindowAdjuster {
    pub percent: f64,
}

impl WeightedMeasurementWindowAdjuster for PercentWeightedMeasurementWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedMeasurementWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedMeasurementWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedMeasurementWindowAdjuster for TieredWeightedMeasurementWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedMeasurementWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted measurement window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedMeasurementWindowEngine {
    pub steps: Vec<Rc<dyn WeightedMeasurementWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedMeasurementWindowEngine {
    pub fn compute(&mut self, item: &WeightedMeasurementWindow) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedMeasurementWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_measurement_window_engine(flags: &flags::Registry) -> WeightedMeasurementWindowEngine {
    WeightedMeasurementWindowEngine {
        steps: vec![
            Rc::new(MinimumWeightedMeasurementWindowAdjuster { minimum: 12.5 })
                as Rc<dyn WeightedMeasurementWindowAdjuster>,
            Rc::new(RoundingWeightedMeasurementWindowAdjuster { step: 0.5 }),
            Rc::new(WeightedMeasurementWindowRotationDaysOffsetAdjuster { weight: 1.55 }),
            Rc::new(FlaggedWeightedMeasurementWindowAdjuster {
                flags: flags.clone(),
                flag: "weighted-measurement-window-capped".to_string(),
                percent: -8.7,
            }),
            Rc::new(PercentWeightedMeasurementWindowAdjuster { percent: 15.6 }),
            Rc::new(TieredWeightedMeasurementWindowAdjuster {
                bounds: vec![1715.0, 6423.0, 7692.0, 9188.0],
                rates: vec![1.17, 1.01, 1.13, 1.05],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 94949.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_measurement_window_engine(&flags::new_registry());
        let item = new_weighted_measurement_window("code-933", "title-971");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
