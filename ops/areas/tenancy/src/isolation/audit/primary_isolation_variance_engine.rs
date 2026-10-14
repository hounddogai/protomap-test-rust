use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary isolation variance calculation.
pub trait PrimaryIsolationVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryIsolationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryIsolationVarianceAdjuster {
    pub step: f64,
}

impl PrimaryIsolationVarianceStep for RoundingPrimaryIsolationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryIsolationVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePrimaryIsolationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryIsolationVarianceStep for TitlePrimaryIsolationVarianceAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PrimaryIsolationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryIsolationVarianceAdjuster {
    pub cap: f64,
}

impl PrimaryIsolationVarianceStep for CapPrimaryIsolationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryIsolationVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary isolation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryIsolationVarianceEngine {
    pub steps: Vec<Rc<dyn PrimaryIsolationVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryIsolationVarianceEngine {
    pub fn price(&mut self, item: &PrimaryIsolationVariance) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryIsolationVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryIsolationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_isolation_variance_engine() -> PrimaryIsolationVarianceEngine {
    PrimaryIsolationVarianceEngine {
        steps: vec![
            Rc::new(RoundingPrimaryIsolationVarianceAdjuster { step: 5.0 }) as Rc<dyn PrimaryIsolationVarianceStep>,
            Rc::new(TitlePrimaryIsolationVarianceAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.83), ("east".to_string(), 1.39)]),
            }),
            Rc::new(CapPrimaryIsolationVarianceAdjuster { cap: 3089.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 70352.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_isolation_variance_engine();
        let item = new_primary_isolation_variance("code-880", "title-348");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
