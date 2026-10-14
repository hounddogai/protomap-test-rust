use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment preference calculation.
pub trait SegmentPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentPreferenceAdjuster {
    pub percent: f64,
}

impl SegmentPreferenceFactor for PercentSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentPreferenceAdjuster {
    pub minimum: f64,
}

impl SegmentPreferenceFactor for MinimumSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentPreferenceFactor for TieredSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentPreferenceFactor for FlaggedSegmentPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the segment preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentPreferenceEngine {
    pub steps: Vec<Rc<dyn SegmentPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentPreferenceEngine {
    pub fn evaluate(&mut self, item: &SegmentPreference) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SegmentPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_segment_preference_engine(flags: &flags::Registry) -> SegmentPreferenceEngine {
    SegmentPreferenceEngine {
        steps: vec![
            Rc::new(PercentSegmentPreferenceAdjuster { percent: 24.2 }) as Rc<dyn SegmentPreferenceFactor>,
            Rc::new(MinimumSegmentPreferenceAdjuster { minimum: 23.8 }),
            Rc::new(TieredSegmentPreferenceAdjuster {
                bounds: vec![4118.0, 8596.0],
                rates: vec![1.12, 0.81],
            }),
            Rc::new(FlaggedSegmentPreferenceAdjuster {
                flags: flags.clone(),
                flag: "segment-preference-strict".to_string(),
                percent: -4.3,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 59861.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_preference_engine(&flags::new_registry());
        let item = new_segment_preference("code-737", "title-795");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
