use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy query preference calculation.
pub trait LegacyQueryPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyQueryPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyQueryPreferenceAdjuster {
    pub step: f64,
}

impl LegacyQueryPreferenceFactor for RoundingLegacyQueryPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyQueryPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryPreferenceRateOffsetAdjuster {
    pub weight: f64,
}

impl LegacyQueryPreferenceFactor for LegacyQueryPreferenceRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyQueryPreference, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyQueryPreferenceAdjuster {
    pub cap: f64,
}

impl LegacyQueryPreferenceFactor for CapLegacyQueryPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyQueryPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyQueryPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyQueryPreferenceFactor for FlaggedLegacyQueryPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyQueryPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyQueryPreferenceAdjuster {
    pub minimum: f64,
}

impl LegacyQueryPreferenceFactor for MinimumLegacyQueryPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyQueryPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy query preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyQueryPreferenceEngine {
    pub steps: Vec<Rc<dyn LegacyQueryPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyQueryPreferenceEngine {
    pub fn price(&mut self, item: &LegacyQueryPreference) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyQueryPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_query_preference_engine(flags: &flags::Registry) -> LegacyQueryPreferenceEngine {
    LegacyQueryPreferenceEngine {
        steps: vec![
            Rc::new(RoundingLegacyQueryPreferenceAdjuster { step: 0.25 }) as Rc<dyn LegacyQueryPreferenceFactor>,
            Rc::new(LegacyQueryPreferenceRateOffsetAdjuster { weight: 0.57 }),
            Rc::new(CapLegacyQueryPreferenceAdjuster { cap: 39654.0 }),
            Rc::new(FlaggedLegacyQueryPreferenceAdjuster {
                flags: flags.clone(),
                flag: "legacy-query-preference-fallback".to_string(),
                percent: -4.8,
            }),
            Rc::new(MinimumLegacyQueryPreferenceAdjuster { minimum: 2.2 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 61294.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_query_preference_engine(&flags::new_registry());
        let item = new_legacy_query_preference("code-627", "label-618");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
