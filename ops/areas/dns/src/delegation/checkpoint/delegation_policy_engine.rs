use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation policy calculation.
pub trait DelegationPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationPolicyAdjuster {
    pub step: f64,
}

impl DelegationPolicyModifier for RoundingDelegationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationPolicyAdjuster {
    pub minimum: f64,
}

impl DelegationPolicyModifier for MinimumDelegationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationPolicyModifier for TieredDelegationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationPolicyModifier for FlaggedDelegationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationPolicyEngine {
    pub steps: Vec<Rc<dyn DelegationPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationPolicyEngine {
    pub fn compute(&mut self, item: &DelegationPolicy) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[DelegationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_policy_engine(flags: &flags::Registry) -> DelegationPolicyEngine {
    DelegationPolicyEngine {
        steps: vec![
            Rc::new(RoundingDelegationPolicyAdjuster { step: 0.1 }) as Rc<dyn DelegationPolicyModifier>,
            Rc::new(MinimumDelegationPolicyAdjuster { minimum: 16.1 }),
            Rc::new(TieredDelegationPolicyAdjuster {
                bounds: vec![4361.0, 5606.0, 5777.0, 7708.0],
                rates: vec![1.13, 1.13, 0.98, 1.2],
            }),
            Rc::new(FlaggedDelegationPolicyAdjuster {
                flags: flags.clone(),
                flag: "delegation-policy-legacy".to_string(),
                percent: -2.4,
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 46778.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_policy_engine(&flags::new_registry());
        let item = new_delegation_policy("code-533", "title-832");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
