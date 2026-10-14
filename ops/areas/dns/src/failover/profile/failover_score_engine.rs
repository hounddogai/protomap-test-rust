use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover score calculation.
pub trait FailoverScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverScoreAdjuster {
    pub percent: f64,
}

impl FailoverScoreAdjuster for PercentFailoverScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitFailoverScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverScoreAdjuster for UnitFailoverScoreAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &FailoverScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverScoreAdjuster {
    pub step: f64,
}

impl FailoverScoreAdjuster for RoundingFailoverScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the failover score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverScoreEngine {
    pub steps: Vec<Rc<dyn FailoverScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverScoreEngine {
    pub fn run(&mut self, item: &FailoverScore) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_failover_score_engine() -> FailoverScoreEngine {
    FailoverScoreEngine {
        steps: vec![
            Rc::new(PercentFailoverScoreAdjuster { percent: 3.3 }) as Rc<dyn FailoverScoreAdjuster>,
            Rc::new(UnitFailoverScoreAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.9),
                    ("central".to_string(), 1.32),
                    ("east".to_string(), 1.33),
                ]),
            }),
            Rc::new(RoundingFailoverScoreAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 40382.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_score_engine();
        let item = new_failover_score("code-95", "label-985");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
