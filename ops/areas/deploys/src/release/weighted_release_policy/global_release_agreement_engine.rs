use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global release agreement calculation.
pub trait GlobalReleaseAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReleaseAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeGlobalReleaseAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalReleaseAgreementFactor for ScopeGlobalReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &GlobalReleaseAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReleaseAgreementAdjuster {
    pub minimum: f64,
}

impl GlobalReleaseAgreementFactor for MinimumGlobalReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReleaseAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalReleaseAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalReleaseAgreementFactor for FlaggedGlobalReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalReleaseAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalReleaseAgreementAdjuster {
    pub percent: f64,
}

impl GlobalReleaseAgreementFactor for PercentGlobalReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalReleaseAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAgreementAmountOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReleaseAgreementFactor for GlobalReleaseAgreementAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReleaseAgreement, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the global release agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReleaseAgreementEngine {
    pub steps: Vec<Rc<dyn GlobalReleaseAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReleaseAgreementEngine {
    pub fn evaluate(&mut self, item: &GlobalReleaseAgreement) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReleaseAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_release_agreement_engine(flags: &flags::Registry) -> GlobalReleaseAgreementEngine {
    GlobalReleaseAgreementEngine {
        steps: vec![
            Rc::new(ScopeGlobalReleaseAgreementAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.91)]),
            }) as Rc<dyn GlobalReleaseAgreementFactor>,
            Rc::new(MinimumGlobalReleaseAgreementAdjuster { minimum: 35.9 }),
            Rc::new(FlaggedGlobalReleaseAgreementAdjuster {
                flags: flags.clone(),
                flag: "global-release-agreement-global".to_string(),
                percent: -4.3,
            }),
            Rc::new(PercentGlobalReleaseAgreementAdjuster { percent: -24.9 }),
            Rc::new(GlobalReleaseAgreementAmountOffsetAdjuster { weight: 1.25 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 40521.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_release_agreement_engine(&flags::new_registry());
        let item = new_global_release_agreement("code-335", "label-274");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
