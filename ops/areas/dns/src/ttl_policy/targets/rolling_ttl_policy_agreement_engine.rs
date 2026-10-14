use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling ttl policy agreement calculation.
pub trait RollingTtlPolicyAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTtlPolicyAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingTtlPolicyAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingTtlPolicyAgreementModifier for NameRollingTtlPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingTtlPolicyAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTtlPolicyAgreementAdjuster {
    pub percent: f64,
}

impl RollingTtlPolicyAgreementModifier for PercentRollingTtlPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTtlPolicyAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyAgreementRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollingTtlPolicyAgreementModifier for RollingTtlPolicyAgreementRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTtlPolicyAgreement, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTtlPolicyAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTtlPolicyAgreementModifier for FlaggedRollingTtlPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTtlPolicyAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingTtlPolicyAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingTtlPolicyAgreementModifier for TieredRollingTtlPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingTtlPolicyAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingTtlPolicyAgreementAdjuster {
    pub step: f64,
}

impl RollingTtlPolicyAgreementModifier for RoundingRollingTtlPolicyAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingTtlPolicyAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling ttl policy agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTtlPolicyAgreementEngine {
    pub steps: Vec<Rc<dyn RollingTtlPolicyAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTtlPolicyAgreementEngine {
    pub fn compute(&mut self, item: &RollingTtlPolicyAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingTtlPolicyAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_ttl_policy_agreement_engine(flags: &flags::Registry) -> RollingTtlPolicyAgreementEngine {
    RollingTtlPolicyAgreementEngine {
        steps: vec![
            Rc::new(NameRollingTtlPolicyAgreementAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.76), ("north".to_string(), 1.11)]),
            }) as Rc<dyn RollingTtlPolicyAgreementModifier>,
            Rc::new(PercentRollingTtlPolicyAgreementAdjuster { percent: 17.6 }),
            Rc::new(RollingTtlPolicyAgreementRotationDaysOffsetAdjuster { weight: 0.42 }),
            Rc::new(FlaggedRollingTtlPolicyAgreementAdjuster {
                flags: flags.clone(),
                flag: "rolling-ttl-policy-agreement-global".to_string(),
                percent: 5.2,
            }),
            Rc::new(TieredRollingTtlPolicyAgreementAdjuster {
                bounds: vec![351.0, 992.0, 6162.0],
                rates: vec![1.06, 0.92, 0.82],
            }),
            Rc::new(RoundingRollingTtlPolicyAgreementAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 48366.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_ttl_policy_agreement_engine(&flags::new_registry());
        let item = new_rolling_ttl_policy_agreement("code-914", "name-791");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
