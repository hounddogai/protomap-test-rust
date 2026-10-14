use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress notice calculation.
pub trait EgressNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressNoticeStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl EgressNoticeAdjuster for EgressNoticeStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &EgressNotice, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressNoticeAdjuster {
    pub percent: f64,
}

impl EgressNoticeAdjuster for PercentEgressNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressNoticeAdjuster {
    pub step: f64,
}

impl EgressNoticeAdjuster for RoundingEgressNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the egress notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressNoticeEngine {
    pub steps: Vec<Rc<dyn EgressNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressNoticeEngine {
    pub fn run(&mut self, item: &EgressNotice) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_notice_engine() -> EgressNoticeEngine {
    EgressNoticeEngine {
        steps: vec![
            Rc::new(EgressNoticeStartsAtOffsetAdjuster { weight: 1.84 }) as Rc<dyn EgressNoticeAdjuster>,
            Rc::new(PercentEgressNoticeAdjuster { percent: 15.3 }),
            Rc::new(RoundingEgressNoticeAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 30232.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_notice_engine();
        let item = new_egress_notice("code-968", "name-661");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
