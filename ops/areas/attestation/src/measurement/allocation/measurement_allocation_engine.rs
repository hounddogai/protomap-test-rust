use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement allocation calculation.
pub trait MeasurementAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementAllocationFactor for FlaggedMeasurementAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementAllocationAdjuster {
    pub percent: f64,
}

impl MeasurementAllocationFactor for PercentMeasurementAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAllocationCostOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementAllocationFactor for MeasurementAllocationCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementAllocation, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementAllocationAdjuster {
    pub step: f64,
}

impl MeasurementAllocationFactor for RoundingMeasurementAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementAllocationAdjuster {
    pub cap: f64,
}

impl MeasurementAllocationFactor for CapMeasurementAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the measurement allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementAllocationEngine {
    pub steps: Vec<Rc<dyn MeasurementAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementAllocationEngine {
    pub fn price(&mut self, item: &MeasurementAllocation) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementAllocationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_measurement_allocation_engine(flags: &flags::Registry) -> MeasurementAllocationEngine {
    MeasurementAllocationEngine {
        steps: vec![
            Rc::new(FlaggedMeasurementAllocationAdjuster {
                flags: flags.clone(),
                flag: "measurement-allocation-seasonal".to_string(),
                percent: -4.7,
            }) as Rc<dyn MeasurementAllocationFactor>,
            Rc::new(PercentMeasurementAllocationAdjuster { percent: 21.8 }),
            Rc::new(MeasurementAllocationCostOffsetAdjuster { weight: 0.61 }),
            Rc::new(RoundingMeasurementAllocationAdjuster { step: 0.25 }),
            Rc::new(CapMeasurementAllocationAdjuster { cap: 18396.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 26099.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_allocation_engine(&flags::new_registry());
        let item = new_measurement_allocation("code-734", "label-955");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
