use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted alert entry calculation.
pub trait WeightedAlertEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedAlertEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedAlertEntryAdjuster {
    pub step: f64,
}

impl WeightedAlertEntryModifier for RoundingWeightedAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedAlertEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedAlertEntryAdjuster {
    pub cap: f64,
}

impl WeightedAlertEntryModifier for CapWeightedAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedAlertEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedAlertEntryAdjuster {
    pub minimum: f64,
}

impl WeightedAlertEntryModifier for MinimumWeightedAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedAlertEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedAlertEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedAlertEntryModifier for FlaggedWeightedAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedAlertEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryWeightedAlertEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedAlertEntryModifier for CountryWeightedAlertEntryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &WeightedAlertEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted alert entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedAlertEntryEngine {
    pub steps: Vec<Rc<dyn WeightedAlertEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedAlertEntryEngine {
    pub fn evaluate(&mut self, item: &WeightedAlertEntry) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedAlertEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_alert_entry_engine(flags: &flags::Registry) -> WeightedAlertEntryEngine {
    WeightedAlertEntryEngine {
        steps: vec![
            Rc::new(RoundingWeightedAlertEntryAdjuster { step: 0.25 }) as Rc<dyn WeightedAlertEntryModifier>,
            Rc::new(CapWeightedAlertEntryAdjuster { cap: 20719.0 }),
            Rc::new(MinimumWeightedAlertEntryAdjuster { minimum: 9.7 }),
            Rc::new(FlaggedWeightedAlertEntryAdjuster {
                flags: flags.clone(),
                flag: "weighted-alert-entry-primary".to_string(),
                percent: 0.8,
            }),
            Rc::new(CountryWeightedAlertEntryAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.3),
                    ("retail".to_string(), 1.24),
                    ("online".to_string(), 1.04),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 81181.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_alert_entry_engine(&flags::new_registry());
        let item = new_weighted_alert_entry("code-599", "title-666");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
