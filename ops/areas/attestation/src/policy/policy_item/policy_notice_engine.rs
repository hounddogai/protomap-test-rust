use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the policy notice calculation.
pub trait PolicyNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyNoticeAdjuster {
    pub step: f64,
}

impl PolicyNoticeStep for RoundingPolicyNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPolicyNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyNoticeStep for RegionPolicyNoticeAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PolicyNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyNoticeLimitOffsetAdjuster {
    pub weight: f64,
}

impl PolicyNoticeStep for PolicyNoticeLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyNotice, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyNoticeAdjuster {
    pub minimum: f64,
}

impl PolicyNoticeStep for MinimumPolicyNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyNoticeAdjuster {
    pub cap: f64,
}

impl PolicyNoticeStep for CapPolicyNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the policy notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyNoticeEngine {
    pub steps: Vec<Rc<dyn PolicyNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyNoticeEngine {
    pub fn evaluate(&mut self, item: &PolicyNotice) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_notice_engine() -> PolicyNoticeEngine {
    PolicyNoticeEngine {
        steps: vec![
            Rc::new(RoundingPolicyNoticeAdjuster { step: 0.25 }) as Rc<dyn PolicyNoticeStep>,
            Rc::new(RegionPolicyNoticeAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.1)]),
            }),
            Rc::new(PolicyNoticeLimitOffsetAdjuster { weight: 1.96 }),
            Rc::new(MinimumPolicyNoticeAdjuster { minimum: 29.0 }),
            Rc::new(CapPolicyNoticeAdjuster { cap: 8437.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 94009.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_notice_engine();
        let item = new_policy_notice("code-670", "title-769");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
