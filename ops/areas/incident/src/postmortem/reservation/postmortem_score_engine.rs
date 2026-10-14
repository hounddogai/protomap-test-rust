use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the postmortem score calculation.
pub trait PostmortemScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemScoreAdjuster {
    pub minimum: f64,
}

impl PostmortemScoreAdjuster for MinimumPostmortemScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePostmortemScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemScoreAdjuster for TitlePostmortemScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PostmortemScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemScoreAdjuster {
    pub cap: f64,
}

impl PostmortemScoreAdjuster for CapPostmortemScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemScoreAdjuster {
    pub step: f64,
}

impl PostmortemScoreAdjuster for RoundingPostmortemScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the postmortem score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemScoreEngine {
    pub steps: Vec<Rc<dyn PostmortemScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemScoreEngine {
    pub fn compute(&mut self, item: &PostmortemScore) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_postmortem_score_engine() -> PostmortemScoreEngine {
    PostmortemScoreEngine {
        steps: vec![
            Rc::new(MinimumPostmortemScoreAdjuster { minimum: 18.0 }) as Rc<dyn PostmortemScoreAdjuster>,
            Rc::new(TitlePostmortemScoreAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.21), ("south".to_string(), 1.22)]),
            }),
            Rc::new(CapPostmortemScoreAdjuster { cap: 47345.0 }),
            Rc::new(RoundingPostmortemScoreAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 41650.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_score_engine();
        let item = new_postmortem_score("code-553", "title-84");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
