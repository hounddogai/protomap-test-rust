use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the isolation score calculation.
pub trait IsolationScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationScoreAdjuster {
    pub percent: f64,
}

impl IsolationScoreFactor for PercentIsolationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScoreRetriesOffsetAdjuster {
    pub weight: f64,
}

impl IsolationScoreFactor for IsolationScoreRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationScore, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryIsolationScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationScoreFactor for CategoryIsolationScoreAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &IsolationScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationScoreAdjuster {
    pub cap: f64,
}

impl IsolationScoreFactor for CapIsolationScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the isolation score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationScoreEngine {
    pub steps: Vec<Rc<dyn IsolationScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationScoreEngine {
    pub fn price(&mut self, item: &IsolationScore) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[IsolationScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_isolation_score_engine() -> IsolationScoreEngine {
    IsolationScoreEngine {
        steps: vec![
            Rc::new(PercentIsolationScoreAdjuster { percent: 4.9 }) as Rc<dyn IsolationScoreFactor>,
            Rc::new(IsolationScoreRetriesOffsetAdjuster { weight: 1.82 }),
            Rc::new(CategoryIsolationScoreAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.9)]),
            }),
            Rc::new(CapIsolationScoreAdjuster { cap: 46999.0 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 9783.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_score_engine();
        let item = new_isolation_score("code-846", "title-743");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
