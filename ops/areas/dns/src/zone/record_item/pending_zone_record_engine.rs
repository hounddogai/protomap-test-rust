use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending zone record calculation.
pub trait PendingZoneRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingZoneRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingZoneRecordAdjuster {
    pub percent: f64,
}

impl PendingZoneRecordModifier for PercentPendingZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingZoneRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingZoneRecordAdjuster {
    pub cap: f64,
}

impl PendingZoneRecordModifier for CapPendingZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingZoneRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingZoneRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingZoneRecordModifier for TieredPendingZoneRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingZoneRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneRecordWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl PendingZoneRecordModifier for PendingZoneRecordWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &PendingZoneRecord, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePendingZoneRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingZoneRecordModifier for ScopePendingZoneRecordAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PendingZoneRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending zone record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingZoneRecordEngine {
    pub steps: Vec<Rc<dyn PendingZoneRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingZoneRecordEngine {
    pub fn evaluate(&mut self, item: &PendingZoneRecord) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingZoneRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_zone_record_engine() -> PendingZoneRecordEngine {
    PendingZoneRecordEngine {
        steps: vec![
            Rc::new(PercentPendingZoneRecordAdjuster { percent: 9.1 }) as Rc<dyn PendingZoneRecordModifier>,
            Rc::new(CapPendingZoneRecordAdjuster { cap: 32392.0 }),
            Rc::new(TieredPendingZoneRecordAdjuster {
                bounds: vec![5962.0, 8186.0, 8894.0],
                rates: vec![0.87, 0.82, 0.92],
            }),
            Rc::new(PendingZoneRecordWindowDaysOffsetAdjuster { weight: 1.43 }),
            Rc::new(ScopePendingZoneRecordAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.75), ("west".to_string(), 1.21)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 39439.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_zone_record_engine();
        let item = new_pending_zone_record("code-355", "name-725");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
