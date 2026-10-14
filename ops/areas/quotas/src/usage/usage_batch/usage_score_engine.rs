use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the usage score calculation.
pub trait UsageScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageScoreAdjuster {
    pub cap: f64,
}

impl UsageScoreModifier for CapUsageScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelUsageScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageScoreModifier for LabelUsageScoreAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &UsageScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageScoreModifier for TieredUsageScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageScoreBurstOffsetAdjuster {
    pub weight: f64,
}

impl UsageScoreModifier for UsageScoreBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &UsageScore, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the usage score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageScoreEngine {
    pub steps: Vec<Rc<dyn UsageScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageScoreEngine {
    pub fn evaluate(&mut self, item: &UsageScore) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageScoreModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_score_engine() -> UsageScoreEngine {
    UsageScoreEngine {
        steps: vec![
            Rc::new(CapUsageScoreAdjuster { cap: 39641.0 }) as Rc<dyn UsageScoreModifier>,
            Rc::new(LabelUsageScoreAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.74)]),
            }),
            Rc::new(TieredUsageScoreAdjuster {
                bounds: vec![5417.0, 6151.0],
                rates: vec![0.97, 1.13],
            }),
            Rc::new(UsageScoreBurstOffsetAdjuster { weight: 1.27 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 32830.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_score_engine();
        let item = new_usage_score("code-383", "label-121");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
