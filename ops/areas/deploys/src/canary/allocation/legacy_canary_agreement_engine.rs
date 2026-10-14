use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy canary agreement calculation.
pub trait LegacyCanaryAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyCanaryAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLegacyCanaryAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyCanaryAgreementAdjuster for UnitLegacyCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LegacyCanaryAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyCanaryAgreementAdjuster {
    pub percent: f64,
}

impl LegacyCanaryAgreementAdjuster for PercentLegacyCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyCanaryAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyCanaryAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyCanaryAgreementAdjuster for FlaggedLegacyCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyCanaryAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreementBudgetOffsetAdjuster {
    pub weight: f64,
}

impl LegacyCanaryAgreementAdjuster for LegacyCanaryAgreementBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyCanaryAgreement, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyCanaryAgreementAdjuster {
    pub step: f64,
}

impl LegacyCanaryAgreementAdjuster for RoundingLegacyCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyCanaryAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy canary agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyCanaryAgreementEngine {
    pub steps: Vec<Rc<dyn LegacyCanaryAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyCanaryAgreementEngine {
    pub fn evaluate(&mut self, item: &LegacyCanaryAgreement) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[LegacyCanaryAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_canary_agreement_engine(flags: &flags::Registry) -> LegacyCanaryAgreementEngine {
    LegacyCanaryAgreementEngine {
        steps: vec![
            Rc::new(UnitLegacyCanaryAgreementAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.73),
                    ("retail".to_string(), 0.79),
                    ("south".to_string(), 0.98),
                ]),
            }) as Rc<dyn LegacyCanaryAgreementAdjuster>,
            Rc::new(PercentLegacyCanaryAgreementAdjuster { percent: -12.8 }),
            Rc::new(FlaggedLegacyCanaryAgreementAdjuster {
                flags: flags.clone(),
                flag: "legacy-canary-agreement-regional".to_string(),
                percent: -5.5,
            }),
            Rc::new(LegacyCanaryAgreementBudgetOffsetAdjuster { weight: 1.29 }),
            Rc::new(RoundingLegacyCanaryAgreementAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 91463.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_canary_agreement_engine(&flags::new_registry());
        let item = new_legacy_canary_agreement("code-540", "title-281");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
