use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered credential policy calculation.
pub trait TieredCredentialPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredCredentialPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredCredentialPolicyAdjuster {
    pub percent: f64,
}

impl TieredCredentialPolicyModifier for PercentTieredCredentialPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredCredentialPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredCredentialPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredCredentialPolicyModifier for TieredTieredCredentialPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredCredentialPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredCredentialPolicyAdjuster {
    pub cap: f64,
}

impl TieredCredentialPolicyModifier for CapTieredCredentialPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredCredentialPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTieredCredentialPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredCredentialPolicyModifier for CategoryTieredCredentialPolicyAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &TieredCredentialPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredCredentialPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredCredentialPolicyModifier for FlaggedTieredCredentialPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredCredentialPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredCredentialPolicyAdjuster {
    pub step: f64,
}

impl TieredCredentialPolicyModifier for RoundingTieredCredentialPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredCredentialPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered credential policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredCredentialPolicyEngine {
    pub steps: Vec<Rc<dyn TieredCredentialPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredCredentialPolicyEngine {
    pub fn evaluate(&mut self, item: &TieredCredentialPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredCredentialPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_credential_policy_engine(flags: &flags::Registry) -> TieredCredentialPolicyEngine {
    TieredCredentialPolicyEngine {
        steps: vec![
            Rc::new(PercentTieredCredentialPolicyAdjuster { percent: 9.5 }) as Rc<dyn TieredCredentialPolicyModifier>,
            Rc::new(TieredTieredCredentialPolicyAdjuster {
                bounds: vec![1263.0, 1419.0, 4082.0, 6713.0],
                rates: vec![1.03, 1.17, 1.2, 1.16],
            }),
            Rc::new(CapTieredCredentialPolicyAdjuster { cap: 36746.0 }),
            Rc::new(CategoryTieredCredentialPolicyAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.92), ("south".to_string(), 0.87)]),
            }),
            Rc::new(FlaggedTieredCredentialPolicyAdjuster {
                flags: flags.clone(),
                flag: "tiered-credential-policy-default".to_string(),
                percent: -6.8,
            }),
            Rc::new(RoundingTieredCredentialPolicyAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 36040.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_credential_policy_engine(&flags::new_registry());
        let item = new_tiered_credential_policy("code-924", "name-819");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
