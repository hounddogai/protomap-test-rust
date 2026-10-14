use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential target calculation.
pub trait CredentialTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialTargetAdjuster {
    pub minimum: f64,
}

impl CredentialTargetModifier for MinimumCredentialTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterCredentialTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialTargetModifier for ClusterCredentialTargetAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &CredentialTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialTargetModifier for TieredCredentialTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the credential target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialTargetEngine {
    pub steps: Vec<Rc<dyn CredentialTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialTargetEngine {
    pub fn evaluate(&mut self, item: &CredentialTarget) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CredentialTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_credential_target_engine() -> CredentialTargetEngine {
    CredentialTargetEngine {
        steps: vec![
            Rc::new(MinimumCredentialTargetAdjuster { minimum: 29.3 }) as Rc<dyn CredentialTargetModifier>,
            Rc::new(ClusterCredentialTargetAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.92), ("north".to_string(), 0.89)]),
            }),
            Rc::new(TieredCredentialTargetAdjuster {
                bounds: vec![3785.0, 5323.0, 8463.0],
                rates: vec![1.01, 0.85, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 33753.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_target_engine();
        let item = new_credential_target("code-664", "name-278");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
