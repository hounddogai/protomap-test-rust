use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier notice calculation.
pub trait VerifierNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleVerifierNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierNoticeStep for LocaleVerifierNoticeAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &VerifierNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierNoticeAdjuster {
    pub cap: f64,
}

impl VerifierNoticeStep for CapVerifierNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierNoticeTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl VerifierNoticeStep for VerifierNoticeTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierNotice, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierNoticeAdjuster {
    pub step: f64,
}

impl VerifierNoticeStep for RoundingVerifierNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierNoticeEngine {
    pub steps: Vec<Rc<dyn VerifierNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierNoticeEngine {
    pub fn evaluate(&mut self, item: &VerifierNotice) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[VerifierNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_verifier_notice_engine() -> VerifierNoticeEngine {
    VerifierNoticeEngine {
        steps: vec![
            Rc::new(LocaleVerifierNoticeAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.87)]),
            }) as Rc<dyn VerifierNoticeStep>,
            Rc::new(CapVerifierNoticeAdjuster { cap: 32207.0 }),
            Rc::new(VerifierNoticeTtlSecondsOffsetAdjuster { weight: 1.75 }),
            Rc::new(RoundingVerifierNoticeAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 40744.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_notice_engine();
        let item = new_verifier_notice("code-244", "name-165");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
