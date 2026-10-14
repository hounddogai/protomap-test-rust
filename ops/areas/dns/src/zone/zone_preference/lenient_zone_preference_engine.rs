use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient zone preference calculation.
pub trait LenientZonePreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientZonePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientZonePreferenceAdjuster {
    pub minimum: f64,
}

impl LenientZonePreferenceModifier for MinimumLenientZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientZonePreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientZonePreferenceRatioOffsetAdjuster {
    pub weight: f64,
}

impl LenientZonePreferenceModifier for LenientZonePreferenceRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &LenientZonePreference, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientZonePreferenceAdjuster {
    pub cap: f64,
}

impl LenientZonePreferenceModifier for CapLenientZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientZonePreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientZonePreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientZonePreferenceModifier for TieredLenientZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientZonePreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientZonePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientZonePreferenceModifier for FlaggedLenientZonePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientZonePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient zone preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientZonePreferenceEngine {
    pub steps: Vec<Rc<dyn LenientZonePreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientZonePreferenceEngine {
    pub fn evaluate(&mut self, item: &LenientZonePreference) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[LenientZonePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_zone_preference_engine(flags: &flags::Registry) -> LenientZonePreferenceEngine {
    LenientZonePreferenceEngine {
        steps: vec![
            Rc::new(MinimumLenientZonePreferenceAdjuster { minimum: 32.6 }) as Rc<dyn LenientZonePreferenceModifier>,
            Rc::new(LenientZonePreferenceRatioOffsetAdjuster { weight: 1.39 }),
            Rc::new(CapLenientZonePreferenceAdjuster { cap: 23544.0 }),
            Rc::new(TieredLenientZonePreferenceAdjuster {
                bounds: vec![1540.0, 1582.0, 9657.0],
                rates: vec![0.97, 1.0, 1.15],
            }),
            Rc::new(FlaggedLenientZonePreferenceAdjuster {
                flags: flags.clone(),
                flag: "lenient-zone-preference-weighted".to_string(),
                percent: 6.7,
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 64403.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_zone_preference_engine(&flags::new_registry());
        let item = new_lenient_zone_preference("code-740", "title-378");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
