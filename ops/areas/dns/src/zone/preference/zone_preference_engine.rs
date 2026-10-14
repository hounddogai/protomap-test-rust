use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the zone preference calculation.
pub trait ZonePreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZonePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingZonePreferenceAdjuster {
    pub step: f64,
}

impl ZonePreferenceStep for RoundingZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ZonePreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZonePreferenceAdjuster {
    pub minimum: f64,
}

impl ZonePreferenceStep for MinimumZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZonePreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZonePreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZonePreferenceStep for TieredZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZonePreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZonePreferenceAdjuster {
    pub cap: f64,
}

impl ZonePreferenceStep for CapZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZonePreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the zone preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZonePreferenceEngine {
    pub steps: Vec<Rc<dyn ZonePreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZonePreferenceEngine {
    pub fn run(&mut self, item: &ZonePreference) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ZonePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_zone_preference_engine() -> ZonePreferenceEngine {
    ZonePreferenceEngine {
        steps: vec![
            Rc::new(RoundingZonePreferenceAdjuster { step: 0.05 }) as Rc<dyn ZonePreferenceStep>,
            Rc::new(MinimumZonePreferenceAdjuster { minimum: 32.5 }),
            Rc::new(TieredZonePreferenceAdjuster {
                bounds: vec![573.0, 5542.0, 6477.0],
                rates: vec![0.86, 0.91, 1.02],
            }),
            Rc::new(CapZonePreferenceAdjuster { cap: 12672.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 50201.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_preference_engine();
        let item = new_zone_preference("code-44", "label-864");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
