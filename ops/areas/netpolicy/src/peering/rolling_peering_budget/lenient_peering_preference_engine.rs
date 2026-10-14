use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient peering preference calculation.
pub trait LenientPeeringPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPeeringPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeLenientPeeringPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientPeeringPreferenceStep for ScopeLenientPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &LenientPeeringPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPeeringPreferenceAdjuster {
    pub minimum: f64,
}

impl LenientPeeringPreferenceStep for MinimumLenientPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPeeringPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientPeeringPreferenceAdjuster {
    pub cap: f64,
}

impl LenientPeeringPreferenceStep for CapLenientPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientPeeringPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreferenceAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl LenientPeeringPreferenceStep for LenientPeeringPreferenceAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &LenientPeeringPreference, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientPeeringPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientPeeringPreferenceStep for FlaggedLenientPeeringPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientPeeringPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient peering preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPeeringPreferenceEngine {
    pub steps: Vec<Rc<dyn LenientPeeringPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPeeringPreferenceEngine {
    pub fn compute(&mut self, item: &LenientPeeringPreference) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientPeeringPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_peering_preference_engine(flags: &flags::Registry) -> LenientPeeringPreferenceEngine {
    LenientPeeringPreferenceEngine {
        steps: vec![
            Rc::new(ScopeLenientPeeringPreferenceAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.83)]),
            }) as Rc<dyn LenientPeeringPreferenceStep>,
            Rc::new(MinimumLenientPeeringPreferenceAdjuster { minimum: 29.8 }),
            Rc::new(CapLenientPeeringPreferenceAdjuster { cap: 48178.0 }),
            Rc::new(LenientPeeringPreferenceAttemptsOffsetAdjuster { weight: 1.03 }),
            Rc::new(FlaggedLenientPeeringPreferenceAdjuster {
                flags: flags.clone(),
                flag: "lenient-peering-preference-tiered".to_string(),
                percent: 6.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 75314.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_peering_preference_engine(&flags::new_registry());
        let item = new_lenient_peering_preference("code-453", "label-818");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
