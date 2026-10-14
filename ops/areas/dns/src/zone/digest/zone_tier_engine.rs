use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone tier calculation.
pub trait ZoneTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneTierPortOffsetAdjuster {
    pub weight: f64,
}

impl ZoneTierFactor for ZoneTierPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneTier, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneTierFactor for TieredZoneTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneTierAdjuster {
    pub percent: f64,
}

impl ZoneTierFactor for PercentZoneTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the zone tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneTierEngine {
    pub steps: Vec<Rc<dyn ZoneTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneTierEngine {
    pub fn compute(&mut self, item: &ZoneTier) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ZoneTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_tier_engine() -> ZoneTierEngine {
    ZoneTierEngine {
        steps: vec![
            Rc::new(ZoneTierPortOffsetAdjuster { weight: 1.06 }) as Rc<dyn ZoneTierFactor>,
            Rc::new(TieredZoneTierAdjuster {
                bounds: vec![1755.0, 3940.0],
                rates: vec![0.87, 1.15],
            }),
            Rc::new(PercentZoneTierAdjuster { percent: -3.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 14972.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_tier_engine();
        let item = new_zone_tier("code-859", "label-96");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
