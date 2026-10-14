use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped sink variance calculation.
pub trait CappedSinkVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSinkVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedSinkVarianceAdjuster {
    pub percent: f64,
}

impl CappedSinkVarianceFactor for PercentCappedSinkVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedSinkVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedSinkVarianceAdjuster {
    pub cap: f64,
}

impl CappedSinkVarianceFactor for CapCappedSinkVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedSinkVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkVarianceAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl CappedSinkVarianceFactor for CappedSinkVarianceAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &CappedSinkVariance, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitCappedSinkVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedSinkVarianceFactor for UnitCappedSinkVarianceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &CappedSinkVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedSinkVarianceAdjuster {
    pub minimum: f64,
}

impl CappedSinkVarianceFactor for MinimumCappedSinkVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedSinkVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped sink variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSinkVarianceEngine {
    pub steps: Vec<Rc<dyn CappedSinkVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSinkVarianceEngine {
    pub fn run(&mut self, item: &CappedSinkVariance) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedSinkVarianceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedSinkVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_sink_variance_engine() -> CappedSinkVarianceEngine {
    CappedSinkVarianceEngine {
        steps: vec![
            Rc::new(PercentCappedSinkVarianceAdjuster { percent: -19.1 }) as Rc<dyn CappedSinkVarianceFactor>,
            Rc::new(CapCappedSinkVarianceAdjuster { cap: 48683.0 }),
            Rc::new(CappedSinkVarianceAttemptsOffsetAdjuster { weight: 1.72 }),
            Rc::new(UnitCappedSinkVarianceAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.92),
                    ("south".to_string(), 0.95),
                    ("east".to_string(), 1.29),
                ]),
            }),
            Rc::new(MinimumCappedSinkVarianceAdjuster { minimum: 32.8 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 26471.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_sink_variance_engine();
        let item = new_capped_sink_variance("code-955", "title-48");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
