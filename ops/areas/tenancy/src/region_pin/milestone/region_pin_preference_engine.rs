use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the region pin preference calculation.
pub trait RegionPinPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionPinPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPreferenceCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionPinPreferenceAdjuster for RegionPinPreferenceCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionPinPreference, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionPinPreferenceAdjuster {
    pub minimum: f64,
}

impl RegionPinPreferenceAdjuster for MinimumRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionPinPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRegionPinPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionPinPreferenceAdjuster for TitleRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RegionPinPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionPinPreferenceAdjuster for TieredRegionPinPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionPinPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the region pin preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionPinPreferenceEngine {
    pub steps: Vec<Rc<dyn RegionPinPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionPinPreferenceEngine {
    pub fn price(&mut self, item: &RegionPinPreference) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionPinPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_region_pin_preference_engine() -> RegionPinPreferenceEngine {
    RegionPinPreferenceEngine {
        steps: vec![
            Rc::new(RegionPinPreferenceCreatedAtOffsetAdjuster { weight: 0.34 }) as Rc<dyn RegionPinPreferenceAdjuster>,
            Rc::new(MinimumRegionPinPreferenceAdjuster { minimum: 31.7 }),
            Rc::new(TitleRegionPinPreferenceAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.78),
                    ("retail".to_string(), 0.93),
                    ("west".to_string(), 1.25),
                ]),
            }),
            Rc::new(TieredRegionPinPreferenceAdjuster {
                bounds: vec![2532.0, 6587.0, 8617.0],
                rates: vec![0.82, 0.95, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 61187.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_region_pin_preference_engine();
        let item = new_region_pin_preference("code-263", "title-158");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
