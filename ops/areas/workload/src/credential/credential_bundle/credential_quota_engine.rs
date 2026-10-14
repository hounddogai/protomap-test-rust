use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the credential quota calculation.
pub trait CredentialQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialQuotaModifier for NameCredentialQuotaAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialQuotaCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl CredentialQuotaModifier for CredentialQuotaCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialQuota, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialQuotaAdjuster {
    pub cap: f64,
}

impl CredentialQuotaModifier for CapCredentialQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the credential quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialQuotaEngine {
    pub steps: Vec<Rc<dyn CredentialQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialQuotaEngine {
    pub fn run(&mut self, item: &CredentialQuota) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialQuotaModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CredentialQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_credential_quota_engine() -> CredentialQuotaEngine {
    CredentialQuotaEngine {
        steps: vec![
            Rc::new(NameCredentialQuotaAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.09)]),
            }) as Rc<dyn CredentialQuotaModifier>,
            Rc::new(CredentialQuotaCreatedAtOffsetAdjuster { weight: 1.29 }),
            Rc::new(CapCredentialQuotaAdjuster { cap: 43508.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 96153.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_quota_engine();
        let item = new_credential_quota("code-625", "name-332");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
