use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted usage preference calculation.
pub trait WeightedUsagePreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedUsagePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedUsagePreferenceAdjuster {
    pub percent: f64,
}

impl WeightedUsagePreferenceModifier for PercentWeightedUsagePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedUsagePreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreferenceRetriesOffsetAdjuster {
    pub weight: f64,
}

impl WeightedUsagePreferenceModifier for WeightedUsagePreferenceRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedUsagePreference, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedUsagePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedUsagePreferenceModifier for FlaggedWeightedUsagePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedUsagePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedUsagePreferenceAdjuster {
    pub cap: f64,
}

impl WeightedUsagePreferenceModifier for CapWeightedUsagePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedUsagePreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedUsagePreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedUsagePreferenceModifier for TieredWeightedUsagePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedUsagePreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentWeightedUsagePreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedUsagePreferenceModifier for SegmentWeightedUsagePreferenceAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &WeightedUsagePreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted usage preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedUsagePreferenceEngine {
    pub steps: Vec<Rc<dyn WeightedUsagePreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedUsagePreferenceEngine {
    pub fn evaluate(&mut self, item: &WeightedUsagePreference) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedUsagePreferenceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedUsagePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_usage_preference_engine(flags: &flags::Registry) -> WeightedUsagePreferenceEngine {
    WeightedUsagePreferenceEngine {
        steps: vec![
            Rc::new(PercentWeightedUsagePreferenceAdjuster { percent: -18.0 })
                as Rc<dyn WeightedUsagePreferenceModifier>,
            Rc::new(WeightedUsagePreferenceRetriesOffsetAdjuster { weight: 1.47 }),
            Rc::new(FlaggedWeightedUsagePreferenceAdjuster {
                flags: flags.clone(),
                flag: "weighted-usage-preference-batched".to_string(),
                percent: 3.3,
            }),
            Rc::new(CapWeightedUsagePreferenceAdjuster { cap: 35010.0 }),
            Rc::new(TieredWeightedUsagePreferenceAdjuster {
                bounds: vec![2071.0, 2920.0, 5220.0, 7068.0],
                rates: vec![0.95, 1.09, 0.91, 0.97],
            }),
            Rc::new(SegmentWeightedUsagePreferenceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.72)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 70898.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_usage_preference_engine(&flags::new_registry());
        let item = new_weighted_usage_preference("code-782", "title-648");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
