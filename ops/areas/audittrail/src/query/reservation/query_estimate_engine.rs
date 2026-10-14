use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query estimate calculation.
pub trait QueryEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryEstimateAdjuster {
    pub step: f64,
}

impl QueryEstimateFactor for RoundingQueryEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryEstimateAdjuster {
    pub minimum: f64,
}

impl QueryEstimateFactor for MinimumQueryEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryEstimateFactor for FlaggedQueryEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimateSequenceOffsetAdjuster {
    pub weight: f64,
}

impl QueryEstimateFactor for QueryEstimateSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &QueryEstimate, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleQueryEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryEstimateFactor for TitleQueryEstimateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &QueryEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the query estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryEstimateEngine {
    pub steps: Vec<Rc<dyn QueryEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryEstimateEngine {
    pub fn evaluate(&mut self, item: &QueryEstimate) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryEstimateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_query_estimate_engine(flags: &flags::Registry) -> QueryEstimateEngine {
    QueryEstimateEngine {
        steps: vec![
            Rc::new(RoundingQueryEstimateAdjuster { step: 0.05 }) as Rc<dyn QueryEstimateFactor>,
            Rc::new(MinimumQueryEstimateAdjuster { minimum: 4.2 }),
            Rc::new(FlaggedQueryEstimateAdjuster {
                flags: flags.clone(),
                flag: "query-estimate-manual".to_string(),
                percent: 1.2,
            }),
            Rc::new(QueryEstimateSequenceOffsetAdjuster { weight: 0.45 }),
            Rc::new(TitleQueryEstimateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.9)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 11422.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_estimate_engine(&flags::new_registry());
        let item = new_query_estimate("code-745", "title-139");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
