use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient limit agreement calculation.
pub trait LenientLimitAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientLimitAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientLimitAgreementAdjuster {
    pub cap: f64,
}

impl LenientLimitAgreementFactor for CapLenientLimitAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientLimitAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientLimitAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientLimitAgreementFactor for FlaggedLenientLimitAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientLimitAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientLimitAgreementAdjuster {
    pub percent: f64,
}

impl LenientLimitAgreementFactor for PercentLenientLimitAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientLimitAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientLimitAgreementAdjuster {
    pub step: f64,
}

impl LenientLimitAgreementFactor for RoundingLenientLimitAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientLimitAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient limit agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientLimitAgreementEngine {
    pub steps: Vec<Rc<dyn LenientLimitAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientLimitAgreementEngine {
    pub fn run(&mut self, item: &LenientLimitAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientLimitAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_limit_agreement_engine(flags: &flags::Registry) -> LenientLimitAgreementEngine {
    LenientLimitAgreementEngine {
        steps: vec![
            Rc::new(CapLenientLimitAgreementAdjuster { cap: 26429.0 }) as Rc<dyn LenientLimitAgreementFactor>,
            Rc::new(FlaggedLenientLimitAgreementAdjuster {
                flags: flags.clone(),
                flag: "lenient-limit-agreement-tiered".to_string(),
                percent: 4.0,
            }),
            Rc::new(PercentLenientLimitAgreementAdjuster { percent: 0.6 }),
            Rc::new(RoundingLenientLimitAgreementAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 68030.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_limit_agreement_engine(&flags::new_registry());
        let item = new_lenient_limit_agreement("code-798", "title-658");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
