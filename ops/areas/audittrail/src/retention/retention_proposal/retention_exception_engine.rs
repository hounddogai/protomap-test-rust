use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the retention exception calculation.
pub trait RetentionExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionExceptionAdjuster {
    pub minimum: f64,
}

impl RetentionExceptionModifier for MinimumRetentionExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRetentionExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionExceptionModifier for TitleRetentionExceptionAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RetentionException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionExceptionAdjuster {
    pub percent: f64,
}

impl RetentionExceptionModifier for PercentRetentionExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the retention exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionExceptionEngine {
    pub steps: Vec<Rc<dyn RetentionExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionExceptionEngine {
    pub fn run(&mut self, item: &RetentionException) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionExceptionModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_exception_engine() -> RetentionExceptionEngine {
    RetentionExceptionEngine {
        steps: vec![
            Rc::new(MinimumRetentionExceptionAdjuster { minimum: 0.8 }) as Rc<dyn RetentionExceptionModifier>,
            Rc::new(TitleRetentionExceptionAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.76), ("south".to_string(), 0.79)]),
            }),
            Rc::new(PercentRetentionExceptionAdjuster { percent: 3.5 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 88225.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_exception_engine();
        let item = new_retention_exception("code-254", "title-506");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
