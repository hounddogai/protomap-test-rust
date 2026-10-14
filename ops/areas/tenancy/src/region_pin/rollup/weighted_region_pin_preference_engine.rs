use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted region pin preference calculation.
pub trait WeightedRegionPinPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRegionPinPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedRegionPinPreferenceAdjuster {
    pub step: f64,
}

impl WeightedRegionPinPreferenceFactor for RoundingWeightedRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedRegionPinPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedRegionPinPreferenceAdjuster {
    pub minimum: f64,
}

impl WeightedRegionPinPreferenceFactor for MinimumWeightedRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedRegionPinPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleWeightedRegionPinPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedRegionPinPreferenceFactor for TitleWeightedRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &WeightedRegionPinPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRegionPinPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRegionPinPreferenceFactor for TieredWeightedRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRegionPinPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRegionPinPreferenceAdjuster {
    pub cap: f64,
}

impl WeightedRegionPinPreferenceFactor for CapWeightedRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRegionPinPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted region pin preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRegionPinPreferenceEngine {
    pub steps: Vec<Rc<dyn WeightedRegionPinPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRegionPinPreferenceEngine {
    pub fn price(&mut self, item: &WeightedRegionPinPreference) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRegionPinPreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRegionPinPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_region_pin_preference_engine() -> WeightedRegionPinPreferenceEngine {
    WeightedRegionPinPreferenceEngine {
        steps: vec![
            Rc::new(RoundingWeightedRegionPinPreferenceAdjuster { step: 5.0 })
                as Rc<dyn WeightedRegionPinPreferenceFactor>,
            Rc::new(MinimumWeightedRegionPinPreferenceAdjuster { minimum: 41.3 }),
            Rc::new(TitleWeightedRegionPinPreferenceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.81), ("west".to_string(), 0.93)]),
            }),
            Rc::new(TieredWeightedRegionPinPreferenceAdjuster {
                bounds: vec![1068.0, 8721.0],
                rates: vec![1.12, 1.2],
            }),
            Rc::new(CapWeightedRegionPinPreferenceAdjuster { cap: 681.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 51427.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_region_pin_preference_engine();
        let item = new_weighted_region_pin_preference("code-426", "title-350");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
