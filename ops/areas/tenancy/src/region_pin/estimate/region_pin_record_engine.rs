use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the region pin record calculation.
pub trait RegionPinRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionPinRecordAdjuster {
    pub cap: f64,
}

impl RegionPinRecordAdjuster for CapRegionPinRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionPinRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinRecordAdjuster for TieredRegionPinRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinRecordAdjuster {
    pub minimum: f64,
}

impl RegionPinRecordAdjuster for MinimumRegionPinRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRecordRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinRecordAdjuster for RegionPinRecordRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinRecord, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionPinRecordAdjuster {
    pub step: f64,
}

impl RegionPinRecordAdjuster for RoundingRegionPinRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionPinRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRegionPinRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinRecordAdjuster for LabelRegionPinRecordAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RegionPinRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the region pin record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinRecordEngine {
    pub steps: Vec<Rc<dyn RegionPinRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinRecordEngine {
    pub fn evaluate(&mut self, item: &RegionPinRecord) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_region_pin_record_engine() -> RegionPinRecordEngine {
    RegionPinRecordEngine {
        steps: vec![
            Rc::new(CapRegionPinRecordAdjuster { cap: 17402.0 }) as Rc<dyn RegionPinRecordAdjuster>,
            Rc::new(TieredRegionPinRecordAdjuster {
                bounds: vec![6230.0, 6560.0, 9738.0],
                rates: vec![0.81, 1.03, 0.89],
            }),
            Rc::new(MinimumRegionPinRecordAdjuster { minimum: 36.0 }),
            Rc::new(RegionPinRecordRotationDaysOffsetAdjuster { weight: 0.77 }),
            Rc::new(RoundingRegionPinRecordAdjuster { step: 1.0 }),
            Rc::new(LabelRegionPinRecordAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.95),
                    ("central".to_string(), 0.92),
                    ("online".to_string(), 1.32),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 84585.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_record_engine();
        let item = new_region_pin_record("code-940", "label-195");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
