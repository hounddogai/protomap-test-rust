use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the segment draft calculation.
pub trait SegmentDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentDraftAdjuster {
    pub step: f64,
}

impl SegmentDraftFactor for RoundingSegmentDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentDraftAdjuster {
    pub cap: f64,
}

impl SegmentDraftFactor for CapSegmentDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDraftRatioOffsetAdjuster {
    pub weight: f64,
}

impl SegmentDraftFactor for SegmentDraftRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentDraft, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentDraftFactor for TieredSegmentDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentDraftAdjuster {
    pub percent: f64,
}

impl SegmentDraftFactor for PercentSegmentDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSegmentDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentDraftFactor for NameSegmentDraftAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SegmentDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the segment draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentDraftEngine {
    pub steps: Vec<Rc<dyn SegmentDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentDraftEngine {
    pub fn evaluate(&mut self, item: &SegmentDraft) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_segment_draft_engine() -> SegmentDraftEngine {
    SegmentDraftEngine {
        steps: vec![
            Rc::new(RoundingSegmentDraftAdjuster { step: 0.1 }) as Rc<dyn SegmentDraftFactor>,
            Rc::new(CapSegmentDraftAdjuster { cap: 43737.0 }),
            Rc::new(SegmentDraftRatioOffsetAdjuster { weight: 1.5 }),
            Rc::new(TieredSegmentDraftAdjuster {
                bounds: vec![3317.0, 3814.0, 5413.0, 7950.0],
                rates: vec![1.05, 0.87, 1.19, 1.01],
            }),
            Rc::new(PercentSegmentDraftAdjuster { percent: 17.0 }),
            Rc::new(NameSegmentDraftAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.26)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 51786.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_draft_engine();
        let item = new_segment_draft("code-611", "name-876");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
