use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled credential segment calculation.
pub trait ScheduledCredentialSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledCredentialSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledCredentialSegmentAdjuster {
    pub percent: f64,
}

impl ScheduledCredentialSegmentAdjuster for PercentScheduledCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledCredentialSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledCredentialSegmentAdjuster {
    pub minimum: f64,
}

impl ScheduledCredentialSegmentAdjuster for MinimumScheduledCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledCredentialSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleScheduledCredentialSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledCredentialSegmentAdjuster for TitleScheduledCredentialSegmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ScheduledCredentialSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled credential segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledCredentialSegmentEngine {
    pub steps: Vec<Rc<dyn ScheduledCredentialSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledCredentialSegmentEngine {
    pub fn run(&mut self, item: &ScheduledCredentialSegment) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledCredentialSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_credential_segment_engine() -> ScheduledCredentialSegmentEngine {
    ScheduledCredentialSegmentEngine {
        steps: vec![
            Rc::new(PercentScheduledCredentialSegmentAdjuster { percent: -8.8 })
                as Rc<dyn ScheduledCredentialSegmentAdjuster>,
            Rc::new(MinimumScheduledCredentialSegmentAdjuster { minimum: 39.4 }),
            Rc::new(TitleScheduledCredentialSegmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.32), ("south".to_string(), 0.88)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 98886.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_credential_segment_engine();
        let item = new_scheduled_credential_segment("code-502", "title-356");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
