use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the page score calculation.
pub trait PageScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPageScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageScoreModifier for ChannelPageScoreAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PageScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageScoreAdjuster {
    pub minimum: f64,
}

impl PageScoreModifier for MinimumPageScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageScoreAdjuster {
    pub cap: f64,
}

impl PageScoreModifier for CapPageScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the page score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageScoreEngine {
    pub steps: Vec<Rc<dyn PageScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageScoreEngine {
    pub fn evaluate(&mut self, item: &PageScore) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageScoreModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_score_engine() -> PageScoreEngine {
    PageScoreEngine {
        steps: vec![
            Rc::new(ChannelPageScoreAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.16),
                    ("north".to_string(), 0.96),
                    ("west".to_string(), 0.81),
                ]),
            }) as Rc<dyn PageScoreModifier>,
            Rc::new(MinimumPageScoreAdjuster { minimum: 0.0 }),
            Rc::new(CapPageScoreAdjuster { cap: 23511.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 58296.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_score_engine();
        let item = new_page_score("code-895", "name-758");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
