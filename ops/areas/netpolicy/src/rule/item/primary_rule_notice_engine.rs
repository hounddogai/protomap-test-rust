use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary rule notice calculation.
pub trait PrimaryRuleNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRuleNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRuleNoticeAdjuster {
    pub cap: f64,
}

impl PrimaryRuleNoticeModifier for CapPrimaryRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRuleNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryRuleNoticeAdjuster {
    pub step: f64,
}

impl PrimaryRuleNoticeModifier for RoundingPrimaryRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryRuleNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRuleNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRuleNoticeModifier for FlaggedPrimaryRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRuleNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryRuleNoticeAdjuster {
    pub percent: f64,
}

impl PrimaryRuleNoticeModifier for PercentPrimaryRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryRuleNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNoticeStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRuleNoticeModifier for PrimaryRuleNoticeStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRuleNotice, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the primary rule notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRuleNoticeEngine {
    pub steps: Vec<Rc<dyn PrimaryRuleNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRuleNoticeEngine {
    pub fn compute(&mut self, item: &PrimaryRuleNotice) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryRuleNoticeModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRuleNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_rule_notice_engine(flags: &flags::Registry) -> PrimaryRuleNoticeEngine {
    PrimaryRuleNoticeEngine {
        steps: vec![
            Rc::new(CapPrimaryRuleNoticeAdjuster { cap: 39129.0 }) as Rc<dyn PrimaryRuleNoticeModifier>,
            Rc::new(RoundingPrimaryRuleNoticeAdjuster { step: 0.1 }),
            Rc::new(FlaggedPrimaryRuleNoticeAdjuster {
                flags: flags.clone(),
                flag: "primary-rule-notice-regional".to_string(),
                percent: 5.6,
            }),
            Rc::new(PercentPrimaryRuleNoticeAdjuster { percent: 5.6 }),
            Rc::new(PrimaryRuleNoticeStartsAtOffsetAdjuster { weight: 1.88 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 36565.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_rule_notice_engine(&flags::new_registry());
        let item = new_primary_rule_notice("code-236", "title-161");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
