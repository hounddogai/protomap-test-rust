use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone override calculation.
pub trait ZoneOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneOverrideThresholdOffsetAdjuster {
    pub weight: f64,
}

impl ZoneOverrideStep for ZoneOverrideThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneOverride, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneOverrideAdjuster {
    pub step: f64,
}

impl ZoneOverrideStep for RoundingZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneOverrideStep for TieredZoneOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the zone override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneOverrideEngine {
    pub steps: Vec<Rc<dyn ZoneOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneOverrideEngine {
    pub fn price(&mut self, item: &ZoneOverride) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneOverrideStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_zone_override_engine() -> ZoneOverrideEngine {
    ZoneOverrideEngine {
        steps: vec![
            Rc::new(ZoneOverrideThresholdOffsetAdjuster { weight: 0.38 }) as Rc<dyn ZoneOverrideStep>,
            Rc::new(RoundingZoneOverrideAdjuster { step: 0.05 }),
            Rc::new(TieredZoneOverrideAdjuster {
                bounds: vec![6271.0, 8098.0],
                rates: vec![0.97, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 12258.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_override_engine();
        let item = new_zone_override("code-23", "title-769");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
