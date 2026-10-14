use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted credential preference calculation.
pub trait WeightedCredentialPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedCredentialPreference, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedCredentialPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedCredentialPreferenceStep for FlaggedWeightedCredentialPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedCredentialPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialPreferencePriorityOffsetAdjuster {
    pub weight: f64,
}

impl WeightedCredentialPreferenceStep for WeightedCredentialPreferencePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedCredentialPreference, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedCredentialPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedCredentialPreferenceStep for TieredWeightedCredentialPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedCredentialPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedCredentialPreferenceAdjuster {
    pub step: f64,
}

impl WeightedCredentialPreferenceStep for RoundingWeightedCredentialPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedCredentialPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedCredentialPreferenceAdjuster {
    pub percent: f64,
}

impl WeightedCredentialPreferenceStep for PercentWeightedCredentialPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedCredentialPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted credential preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedCredentialPreferenceEngine {
    pub steps: Vec<Rc<dyn WeightedCredentialPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedCredentialPreferenceEngine {
    pub fn price(&mut self, item: &WeightedCredentialPreference) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[WeightedCredentialPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_credential_preference_engine(flags: &flags::Registry) -> WeightedCredentialPreferenceEngine {
    WeightedCredentialPreferenceEngine {
        steps: vec![
            Rc::new(FlaggedWeightedCredentialPreferenceAdjuster {
                flags: flags.clone(),
                flag: "weighted-credential-preference-regional".to_string(),
                percent: 4.1,
            }) as Rc<dyn WeightedCredentialPreferenceStep>,
            Rc::new(WeightedCredentialPreferencePriorityOffsetAdjuster { weight: 1.46 }),
            Rc::new(TieredWeightedCredentialPreferenceAdjuster {
                bounds: vec![1247.0, 2978.0, 6803.0],
                rates: vec![1.15, 0.87, 0.98],
            }),
            Rc::new(RoundingWeightedCredentialPreferenceAdjuster { step: 1.0 }),
            Rc::new(PercentWeightedCredentialPreferenceAdjuster { percent: 17.8 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 75388.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_credential_preference_engine(&flags::new_registry());
        let item = new_weighted_credential_preference("code-562", "name-795");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
