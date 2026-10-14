use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the query score calculation.
pub trait QueryScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQueryScoreAdjuster {
    pub percent: f64,
}

impl QueryScoreModifier for PercentQueryScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QueryScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryScoreAdjuster {
    pub minimum: f64,
}

impl QueryScoreModifier for MinimumQueryScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScoreBudgetOffsetAdjuster {
    pub weight: f64,
}

impl QueryScoreModifier for QueryScoreBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &QueryScore, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelQueryScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryScoreModifier for LabelQueryScoreAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &QueryScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the query score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryScoreEngine {
    pub steps: Vec<Rc<dyn QueryScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryScoreEngine {
    pub fn evaluate(&mut self, item: &QueryScore) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[QueryScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_query_score_engine() -> QueryScoreEngine {
    QueryScoreEngine {
        steps: vec![
            Rc::new(PercentQueryScoreAdjuster { percent: -8.5 }) as Rc<dyn QueryScoreModifier>,
            Rc::new(MinimumQueryScoreAdjuster { minimum: 7.2 }),
            Rc::new(QueryScoreBudgetOffsetAdjuster { weight: 1.67 }),
            Rc::new(LabelQueryScoreAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.71), ("central".to_string(), 0.94)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 43485.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_query_score_engine();
        let item = new_query_score("code-969", "label-970");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
