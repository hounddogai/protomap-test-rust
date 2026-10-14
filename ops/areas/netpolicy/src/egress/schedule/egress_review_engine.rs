use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the egress review calculation.
pub trait EgressReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEgressReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressReviewAdjuster for LabelEgressReviewAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EgressReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressReviewGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl EgressReviewAdjuster for EgressReviewGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &EgressReview, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressReviewAdjuster {
    pub percent: f64,
}

impl EgressReviewAdjuster for PercentEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressReviewAdjuster for FlaggedEgressReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressReviewEngine {
    pub steps: Vec<Rc<dyn EgressReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressReviewEngine {
    pub fn run(&mut self, item: &EgressReview) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressReviewAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_review_engine(flags: &flags::Registry) -> EgressReviewEngine {
    EgressReviewEngine {
        steps: vec![
            Rc::new(LabelEgressReviewAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.98), ("south".to_string(), 0.82)]),
            }) as Rc<dyn EgressReviewAdjuster>,
            Rc::new(EgressReviewGraceDaysOffsetAdjuster { weight: 0.95 }),
            Rc::new(PercentEgressReviewAdjuster { percent: -23.2 }),
            Rc::new(FlaggedEgressReviewAdjuster {
                flags: flags.clone(),
                flag: "egress-review-legacy".to_string(),
                percent: 6.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 10705.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_review_engine(&flags::new_registry());
        let item = new_egress_review("code-409", "label-796");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
