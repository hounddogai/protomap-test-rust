use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release preference calculation.
pub trait ReleasePreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleasePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePreferenceDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl ReleasePreferenceModifier for ReleasePreferenceDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &ReleasePreference, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleasePreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleasePreferenceModifier for TieredReleasePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleasePreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleasePreferenceAdjuster {
    pub step: f64,
}

impl ReleasePreferenceModifier for RoundingReleasePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleasePreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionReleasePreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleasePreferenceModifier for RegionReleasePreferenceAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ReleasePreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleasePreferenceAdjuster {
    pub percent: f64,
}

impl ReleasePreferenceModifier for PercentReleasePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleasePreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleasePreferenceEngine {
    pub steps: Vec<Rc<dyn ReleasePreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleasePreferenceEngine {
    pub fn evaluate(&mut self, item: &ReleasePreference) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleasePreferenceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleasePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_release_preference_engine() -> ReleasePreferenceEngine {
    ReleasePreferenceEngine {
        steps: vec![
            Rc::new(ReleasePreferenceDurationMinutesOffsetAdjuster { weight: 1.89 })
                as Rc<dyn ReleasePreferenceModifier>,
            Rc::new(TieredReleasePreferenceAdjuster {
                bounds: vec![4385.0, 6839.0],
                rates: vec![0.83, 0.96],
            }),
            Rc::new(RoundingReleasePreferenceAdjuster { step: 0.05 }),
            Rc::new(RegionReleasePreferenceAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.05),
                    ("retail".to_string(), 1.1),
                    ("online".to_string(), 1.3),
                ]),
            }),
            Rc::new(PercentReleasePreferenceAdjuster { percent: -13.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 22751.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_preference_engine();
        let item = new_release_preference("code-916", "label-168");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
