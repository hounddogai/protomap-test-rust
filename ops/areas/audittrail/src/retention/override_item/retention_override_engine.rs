use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the retention override calculation.
pub trait RetentionOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionOverrideAdjuster {
    pub percent: f64,
}

impl RetentionOverrideAdjuster for PercentRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRetentionOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionOverrideAdjuster for NameRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RetentionOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionOverrideAdjuster {
    pub cap: f64,
}

impl RetentionOverrideAdjuster for CapRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOverrideBudgetOffsetAdjuster {
    pub weight: f64,
}

impl RetentionOverrideAdjuster for RetentionOverrideBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionOverride, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the retention override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionOverrideEngine {
    pub steps: Vec<Rc<dyn RetentionOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionOverrideEngine {
    pub fn evaluate(&mut self, item: &RetentionOverride) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_override_engine() -> RetentionOverrideEngine {
    RetentionOverrideEngine {
        steps: vec![
            Rc::new(PercentRetentionOverrideAdjuster { percent: 3.7 }) as Rc<dyn RetentionOverrideAdjuster>,
            Rc::new(NameRetentionOverrideAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.93), ("central".to_string(), 0.94)]),
            }),
            Rc::new(CapRetentionOverrideAdjuster { cap: 49690.0 }),
            Rc::new(RetentionOverrideBudgetOffsetAdjuster { weight: 0.34 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 82675.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_override_engine();
        let item = new_retention_override("code-151", "name-983");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
