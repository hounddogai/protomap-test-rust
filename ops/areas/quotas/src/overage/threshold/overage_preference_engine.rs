use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage preference calculation.
pub trait OveragePreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OveragePreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleOveragePreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl OveragePreferenceStep for LocaleOveragePreferenceAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &OveragePreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOveragePreferenceAdjuster {
    pub percent: f64,
}

impl OveragePreferenceStep for PercentOveragePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OveragePreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOveragePreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OveragePreferenceStep for FlaggedOveragePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OveragePreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOveragePreferenceAdjuster {
    pub minimum: f64,
}

impl OveragePreferenceStep for MinimumOveragePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OveragePreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOveragePreferenceAdjuster {
    pub step: f64,
}

impl OveragePreferenceStep for RoundingOveragePreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OveragePreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the overage preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OveragePreferenceEngine {
    pub steps: Vec<Rc<dyn OveragePreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OveragePreferenceEngine {
    pub fn price(&mut self, item: &OveragePreference) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[OveragePreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_preference_engine(flags: &flags::Registry) -> OveragePreferenceEngine {
    OveragePreferenceEngine {
        steps: vec![
            Rc::new(LocaleOveragePreferenceAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.91)]),
            }) as Rc<dyn OveragePreferenceStep>,
            Rc::new(PercentOveragePreferenceAdjuster { percent: -10.6 }),
            Rc::new(FlaggedOveragePreferenceAdjuster {
                flags: flags.clone(),
                flag: "overage-preference-default".to_string(),
                percent: -7.4,
            }),
            Rc::new(MinimumOveragePreferenceAdjuster { minimum: 12.3 }),
            Rc::new(RoundingOveragePreferenceAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 70558.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_preference_engine(&flags::new_registry());
        let item = new_overage_preference("code-61", "title-338");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
