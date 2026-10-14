use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential milestone calculation.
pub trait CredentialMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialMilestoneModifier for NameCredentialMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialMilestoneAdjuster {
    pub percent: f64,
}

impl CredentialMilestoneModifier for PercentCredentialMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialMilestoneModifier for TieredCredentialMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the credential milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialMilestoneEngine {
    pub steps: Vec<Rc<dyn CredentialMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialMilestoneEngine {
    pub fn run(&mut self, item: &CredentialMilestone) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CredentialMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_credential_milestone_engine() -> CredentialMilestoneEngine {
    CredentialMilestoneEngine {
        steps: vec![
            Rc::new(NameCredentialMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.95), ("west".to_string(), 0.76)]),
            }) as Rc<dyn CredentialMilestoneModifier>,
            Rc::new(PercentCredentialMilestoneAdjuster { percent: -19.3 }),
            Rc::new(TieredCredentialMilestoneAdjuster {
                bounds: vec![7497.0, 8609.0],
                rates: vec![0.84, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 48760.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_milestone_engine();
        let item = new_credential_milestone("code-698", "name-171");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
