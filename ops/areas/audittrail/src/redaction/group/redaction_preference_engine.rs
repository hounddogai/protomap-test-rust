use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction preference calculation.
pub trait RedactionPreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionPreferenceAdjuster {
    pub minimum: f64,
}

impl RedactionPreferenceModifier for MinimumRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionPreferenceModifier for TieredRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionPreferenceAdjuster {
    pub percent: f64,
}

impl RedactionPreferenceModifier for PercentRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionPreferenceAdjuster {
    pub cap: f64,
}

impl RedactionPreferenceModifier for CapRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRedactionPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionPreferenceModifier for RegionRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RedactionPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionPreferenceModifier for FlaggedRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionPreferenceEngine {
    pub steps: Vec<Rc<dyn RedactionPreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionPreferenceEngine {
    pub fn price(&mut self, item: &RedactionPreference) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RedactionPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_redaction_preference_engine(flags: &flags::Registry) -> RedactionPreferenceEngine {
    RedactionPreferenceEngine {
        steps: vec![
            Rc::new(MinimumRedactionPreferenceAdjuster { minimum: 22.3 }) as Rc<dyn RedactionPreferenceModifier>,
            Rc::new(TieredRedactionPreferenceAdjuster {
                bounds: vec![48.0, 6726.0],
                rates: vec![0.91, 0.86],
            }),
            Rc::new(PercentRedactionPreferenceAdjuster { percent: 16.6 }),
            Rc::new(CapRedactionPreferenceAdjuster { cap: 8853.0 }),
            Rc::new(RegionRedactionPreferenceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.02), ("online".to_string(), 1.16)]),
            }),
            Rc::new(FlaggedRedactionPreferenceAdjuster {
                flags: flags.clone(),
                flag: "redaction-preference-regional".to_string(),
                percent: -4.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 24850.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_preference_engine(&flags::new_registry());
        let item = new_redaction_preference("code-791", "name-264");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
