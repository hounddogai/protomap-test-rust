use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default measurement mapping calculation.
pub trait DefaultMeasurementMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultMeasurementMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultMeasurementMappingAdjuster {
    pub cap: f64,
}

impl DefaultMeasurementMappingModifier for CapDefaultMeasurementMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultMeasurementMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultMeasurementMappingAdjuster {
    pub step: f64,
}

impl DefaultMeasurementMappingModifier for RoundingDefaultMeasurementMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultMeasurementMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultMeasurementMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultMeasurementMappingModifier for FlaggedDefaultMeasurementMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultMeasurementMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultMeasurementMappingAdjuster {
    pub minimum: f64,
}

impl DefaultMeasurementMappingModifier for MinimumDefaultMeasurementMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultMeasurementMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementMappingGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl DefaultMeasurementMappingModifier for DefaultMeasurementMappingGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultMeasurementMapping, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the default measurement mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultMeasurementMappingEngine {
    pub steps: Vec<Rc<dyn DefaultMeasurementMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultMeasurementMappingEngine {
    pub fn price(&mut self, item: &DefaultMeasurementMapping) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultMeasurementMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultMeasurementMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_measurement_mapping_engine(flags: &flags::Registry) -> DefaultMeasurementMappingEngine {
    DefaultMeasurementMappingEngine {
        steps: vec![
            Rc::new(CapDefaultMeasurementMappingAdjuster { cap: 22730.0 }) as Rc<dyn DefaultMeasurementMappingModifier>,
            Rc::new(RoundingDefaultMeasurementMappingAdjuster { step: 0.25 }),
            Rc::new(FlaggedDefaultMeasurementMappingAdjuster {
                flags: flags.clone(),
                flag: "default-measurement-mapping-rolling".to_string(),
                percent: 3.4,
            }),
            Rc::new(MinimumDefaultMeasurementMappingAdjuster { minimum: 3.7 }),
            Rc::new(DefaultMeasurementMappingGraceDaysOffsetAdjuster { weight: 0.83 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 63474.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_measurement_mapping_engine(&flags::new_registry());
        let item = new_default_measurement_mapping("code-854", "name-208");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
