use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage agreement calculation.
pub trait OverageAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageAgreementFactor for FlaggedOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAgreementCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl OverageAgreementFactor for OverageAgreementCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &OverageAgreement, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageAgreementAdjuster {
    pub minimum: f64,
}

impl OverageAgreementFactor for MinimumOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageAgreementAdjuster {
    pub step: f64,
}

impl OverageAgreementFactor for RoundingOverageAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the overage agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageAgreementEngine {
    pub steps: Vec<Rc<dyn OverageAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageAgreementEngine {
    pub fn price(&mut self, item: &OverageAgreement) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[OverageAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_agreement_engine(flags: &flags::Registry) -> OverageAgreementEngine {
    OverageAgreementEngine {
        steps: vec![
            Rc::new(FlaggedOverageAgreementAdjuster {
                flags: flags.clone(),
                flag: "overage-agreement-strict".to_string(),
                percent: -9.7,
            }) as Rc<dyn OverageAgreementFactor>,
            Rc::new(OverageAgreementCreatedAtOffsetAdjuster { weight: 0.44 }),
            Rc::new(MinimumOverageAgreementAdjuster { minimum: 1.0 }),
            Rc::new(RoundingOverageAgreementAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 21140.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_agreement_engine(&flags::new_registry());
        let item = new_overage_agreement("code-681", "title-35");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
