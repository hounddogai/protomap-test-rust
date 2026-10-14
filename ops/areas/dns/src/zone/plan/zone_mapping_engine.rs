use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone mapping calculation.
pub trait ZoneMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneMappingAdjuster {
    pub percent: f64,
}

impl ZoneMappingModifier for PercentZoneMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneMappingModifier for TieredZoneMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZoneMappingAdjuster {
    pub step: f64,
}

impl ZoneMappingModifier for RoundingZoneMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZoneMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneMappingAdjuster {
    pub cap: f64,
}

impl ZoneMappingModifier for CapZoneMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMappingBurstOffsetAdjuster {
    pub weight: f64,
}

impl ZoneMappingModifier for ZoneMappingBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneMapping, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the zone mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneMappingEngine {
    pub steps: Vec<Rc<dyn ZoneMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneMappingEngine {
    pub fn compute(&mut self, item: &ZoneMapping) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_mapping_engine() -> ZoneMappingEngine {
    ZoneMappingEngine {
        steps: vec![
            Rc::new(PercentZoneMappingAdjuster { percent: 16.7 }) as Rc<dyn ZoneMappingModifier>,
            Rc::new(TieredZoneMappingAdjuster {
                bounds: vec![270.0, 5921.0],
                rates: vec![1.05, 1.09],
            }),
            Rc::new(RoundingZoneMappingAdjuster { step: 5.0 }),
            Rc::new(CapZoneMappingAdjuster { cap: 666.0 }),
            Rc::new(ZoneMappingBurstOffsetAdjuster { weight: 1.47 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 11402.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_mapping_engine();
        let item = new_zone_mapping("code-149", "title-752");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
