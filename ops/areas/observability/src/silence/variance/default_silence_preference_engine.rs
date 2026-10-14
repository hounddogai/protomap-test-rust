use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default silence preference calculation.
pub trait DefaultSilencePreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultSilencePreference, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultSilencePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultSilencePreferenceModifier for FlaggedDefaultSilencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultSilencePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultSilencePreferenceAdjuster {
    pub minimum: f64,
}

impl DefaultSilencePreferenceModifier for MinimumDefaultSilencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultSilencePreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultSilencePreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultSilencePreferenceModifier for TieredDefaultSilencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultSilencePreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultSilencePreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultSilencePreferenceModifier for LabelDefaultSilencePreferenceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultSilencePreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultSilencePreferenceAdjuster {
    pub percent: f64,
}

impl DefaultSilencePreferenceModifier for PercentDefaultSilencePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultSilencePreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default silence preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultSilencePreferenceEngine {
    pub steps: Vec<Rc<dyn DefaultSilencePreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultSilencePreferenceEngine {
    pub fn evaluate(&mut self, item: &DefaultSilencePreference) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultSilencePreferenceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultSilencePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_silence_preference_engine(flags: &flags::Registry) -> DefaultSilencePreferenceEngine {
    DefaultSilencePreferenceEngine {
        steps: vec![
            Rc::new(FlaggedDefaultSilencePreferenceAdjuster {
                flags: flags.clone(),
                flag: "default-silence-preference-regional".to_string(),
                percent: -3.7,
            }) as Rc<dyn DefaultSilencePreferenceModifier>,
            Rc::new(MinimumDefaultSilencePreferenceAdjuster { minimum: 11.6 }),
            Rc::new(TieredDefaultSilencePreferenceAdjuster {
                bounds: vec![177.0, 3150.0],
                rates: vec![0.93, 1.11],
            }),
            Rc::new(LabelDefaultSilencePreferenceAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.82)]),
            }),
            Rc::new(PercentDefaultSilencePreferenceAdjuster { percent: -23.4 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 66891.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_silence_preference_engine(&flags::new_registry());
        let item = new_default_silence_preference("code-495", "label-969");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
