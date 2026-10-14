use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the zone record calculation.
pub trait ZoneRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneRecordAdjuster for TieredZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneRecordAdjuster {
    pub cap: f64,
}

impl ZoneRecordAdjuster for CapZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneRecordAdjuster {
    pub minimum: f64,
}

impl ZoneRecordAdjuster for MinimumZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneRecordAdjuster {
    pub percent: f64,
}

impl ZoneRecordAdjuster for PercentZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the zone record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneRecordEngine {
    pub steps: Vec<Rc<dyn ZoneRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneRecordEngine {
    pub fn evaluate(&mut self, item: &ZoneRecord) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_zone_record_engine() -> ZoneRecordEngine {
    ZoneRecordEngine {
        steps: vec![
            Rc::new(TieredZoneRecordAdjuster {
                bounds: vec![1198.0, 6480.0, 7628.0],
                rates: vec![1.15, 0.94, 0.95],
            }) as Rc<dyn ZoneRecordAdjuster>,
            Rc::new(CapZoneRecordAdjuster { cap: 31817.0 }),
            Rc::new(MinimumZoneRecordAdjuster { minimum: 17.8 }),
            Rc::new(PercentZoneRecordAdjuster { percent: 18.2 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 95337.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_record_engine();
        let item = new_zone_record("code-731", "name-622");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
