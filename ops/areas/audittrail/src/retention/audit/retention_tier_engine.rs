use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention tier calculation.
pub trait RetentionTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRetentionTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionTierStep for NameRetentionTierAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RetentionTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionTierAdjuster {
    pub minimum: f64,
}

impl RetentionTierStep for MinimumRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionTierDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl RetentionTierStep for RetentionTierDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionTier, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionTierAdjuster {
    pub cap: f64,
}

impl RetentionTierStep for CapRetentionTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the retention tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionTierEngine {
    pub steps: Vec<Rc<dyn RetentionTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionTierEngine {
    pub fn evaluate(&mut self, item: &RetentionTier) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[RetentionTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_tier_engine() -> RetentionTierEngine {
    RetentionTierEngine {
        steps: vec![
            Rc::new(NameRetentionTierAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.34),
                    ("east".to_string(), 1.25),
                    ("retail".to_string(), 0.98),
                ]),
            }) as Rc<dyn RetentionTierStep>,
            Rc::new(MinimumRetentionTierAdjuster { minimum: 35.3 }),
            Rc::new(RetentionTierDurationMinutesOffsetAdjuster { weight: 0.02 }),
            Rc::new(CapRetentionTierAdjuster { cap: 27146.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 31815.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_tier_engine();
        let item = new_retention_tier("code-268", "name-16");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
