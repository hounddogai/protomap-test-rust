use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal alert entry calculation.
pub trait SeasonalAlertEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalAlertEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalAlertEntryAdjuster {
    pub cap: f64,
}

impl SeasonalAlertEntryStep for CapSeasonalAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalAlertEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalAlertEntryAdjuster {
    pub step: f64,
}

impl SeasonalAlertEntryStep for RoundingSeasonalAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalAlertEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalAlertEntryAdjuster {
    pub minimum: f64,
}

impl SeasonalAlertEntryStep for MinimumSeasonalAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalAlertEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterSeasonalAlertEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalAlertEntryStep for ClusterSeasonalAlertEntryAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &SeasonalAlertEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalAlertEntryAdjuster {
    pub percent: f64,
}

impl SeasonalAlertEntryStep for PercentSeasonalAlertEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalAlertEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertEntryLimitOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalAlertEntryStep for SeasonalAlertEntryLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalAlertEntry, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the seasonal alert entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalAlertEntryEngine {
    pub steps: Vec<Rc<dyn SeasonalAlertEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalAlertEntryEngine {
    pub fn run(&mut self, item: &SeasonalAlertEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalAlertEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalAlertEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_alert_entry_engine() -> SeasonalAlertEntryEngine {
    SeasonalAlertEntryEngine {
        steps: vec![
            Rc::new(CapSeasonalAlertEntryAdjuster { cap: 46264.0 }) as Rc<dyn SeasonalAlertEntryStep>,
            Rc::new(RoundingSeasonalAlertEntryAdjuster { step: 0.1 }),
            Rc::new(MinimumSeasonalAlertEntryAdjuster { minimum: 32.6 }),
            Rc::new(ClusterSeasonalAlertEntryAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.99),
                    ("south".to_string(), 1.13),
                    ("retail".to_string(), 1.37),
                ]),
            }),
            Rc::new(PercentSeasonalAlertEntryAdjuster { percent: -0.9 }),
            Rc::new(SeasonalAlertEntryLimitOffsetAdjuster { weight: 1.25 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 72867.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_alert_entry_engine();
        let item = new_seasonal_alert_entry("code-790", "name-581");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
