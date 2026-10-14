use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook threshold calculation.
pub trait RunbookThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookThresholdAdjuster {
    pub step: f64,
}

impl RunbookThresholdStep for RoundingRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookThresholdAdjuster {
    pub minimum: f64,
}

impl RunbookThresholdStep for MinimumRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookThresholdAdjuster {
    pub percent: f64,
}

impl RunbookThresholdStep for PercentRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookThresholdAdjuster {
    pub cap: f64,
}

impl RunbookThresholdStep for CapRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRunbookThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookThresholdStep for LocaleRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RunbookThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the runbook threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookThresholdEngine {
    pub steps: Vec<Rc<dyn RunbookThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookThresholdEngine {
    pub fn compute(&mut self, item: &RunbookThreshold) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_runbook_threshold_engine() -> RunbookThresholdEngine {
    RunbookThresholdEngine {
        steps: vec![
            Rc::new(RoundingRunbookThresholdAdjuster { step: 0.25 }) as Rc<dyn RunbookThresholdStep>,
            Rc::new(MinimumRunbookThresholdAdjuster { minimum: 2.5 }),
            Rc::new(PercentRunbookThresholdAdjuster { percent: -8.9 }),
            Rc::new(CapRunbookThresholdAdjuster { cap: 24720.0 }),
            Rc::new(LocaleRunbookThresholdAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.11),
                    ("retail".to_string(), 1.09),
                    ("south".to_string(), 1.1),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 93630.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_threshold_engine();
        let item = new_runbook_threshold("code-102", "name-649");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
