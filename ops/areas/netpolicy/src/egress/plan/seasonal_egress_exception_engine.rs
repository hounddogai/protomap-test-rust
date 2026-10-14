use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal egress exception calculation.
pub trait SeasonalEgressExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalEgressException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalEgressExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalEgressExceptionAdjuster for TieredSeasonalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalEgressException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalEgressExceptionAdjuster {
    pub minimum: f64,
}

impl SeasonalEgressExceptionAdjuster for MinimumSeasonalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalEgressException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSeasonalEgressExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalEgressExceptionAdjuster for SourceSeasonalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SeasonalEgressException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalEgressExceptionAdjuster {
    pub cap: f64,
}

impl SeasonalEgressExceptionAdjuster for CapSeasonalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalEgressException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalEgressExceptionAdjuster {
    pub step: f64,
}

impl SeasonalEgressExceptionAdjuster for RoundingSeasonalEgressExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalEgressException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressExceptionRetriesOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalEgressExceptionAdjuster for SeasonalEgressExceptionRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalEgressException, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the seasonal egress exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalEgressExceptionEngine {
    pub steps: Vec<Rc<dyn SeasonalEgressExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalEgressExceptionEngine {
    pub fn compute(&mut self, item: &SeasonalEgressException) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalEgressExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalEgressException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_egress_exception_engine() -> SeasonalEgressExceptionEngine {
    SeasonalEgressExceptionEngine {
        steps: vec![
            Rc::new(TieredSeasonalEgressExceptionAdjuster {
                bounds: vec![1923.0, 7418.0],
                rates: vec![0.82, 1.19],
            }) as Rc<dyn SeasonalEgressExceptionAdjuster>,
            Rc::new(MinimumSeasonalEgressExceptionAdjuster { minimum: 47.1 }),
            Rc::new(SourceSeasonalEgressExceptionAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.13)]),
            }),
            Rc::new(CapSeasonalEgressExceptionAdjuster { cap: 31775.0 }),
            Rc::new(RoundingSeasonalEgressExceptionAdjuster { step: 1.0 }),
            Rc::new(SeasonalEgressExceptionRetriesOffsetAdjuster { weight: 1.37 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 43976.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_egress_exception_engine();
        let item = new_seasonal_egress_exception("code-71", "name-996");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
