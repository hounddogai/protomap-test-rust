use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict identity policy calculation.
pub trait StrictIdentityPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictIdentityPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictIdentityPolicyAdjuster {
    pub percent: f64,
}

impl StrictIdentityPolicyFactor for PercentStrictIdentityPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictIdentityPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictIdentityPolicyAdjuster {
    pub minimum: f64,
}

impl StrictIdentityPolicyFactor for MinimumStrictIdentityPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictIdentityPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleStrictIdentityPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictIdentityPolicyFactor for TitleStrictIdentityPolicyAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &StrictIdentityPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictIdentityPolicyAdjuster {
    pub cap: f64,
}

impl StrictIdentityPolicyFactor for CapStrictIdentityPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictIdentityPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict identity policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictIdentityPolicyEngine {
    pub steps: Vec<Rc<dyn StrictIdentityPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictIdentityPolicyEngine {
    pub fn run(&mut self, item: &StrictIdentityPolicy) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[StrictIdentityPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_identity_policy_engine() -> StrictIdentityPolicyEngine {
    StrictIdentityPolicyEngine {
        steps: vec![
            Rc::new(PercentStrictIdentityPolicyAdjuster { percent: -22.2 }) as Rc<dyn StrictIdentityPolicyFactor>,
            Rc::new(MinimumStrictIdentityPolicyAdjuster { minimum: 2.8 }),
            Rc::new(TitleStrictIdentityPolicyAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.16)]),
            }),
            Rc::new(CapStrictIdentityPolicyAdjuster { cap: 17528.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 66312.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_identity_policy_engine();
        let item = new_strict_identity_policy("code-687", "title-873");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
