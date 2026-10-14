use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the credential claim calculation.
pub trait CredentialClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialClaimAdjuster for TieredCredentialClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialClaimRevisionOffsetAdjuster {
    pub weight: f64,
}

impl CredentialClaimAdjuster for CredentialClaimRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialClaim, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialClaimAdjuster for NameCredentialClaimAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialClaimAdjuster for FlaggedCredentialClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the credential claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialClaimEngine {
    pub steps: Vec<Rc<dyn CredentialClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialClaimEngine {
    pub fn compute(&mut self, item: &CredentialClaim) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[CredentialClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_credential_claim_engine(flags: &flags::Registry) -> CredentialClaimEngine {
    CredentialClaimEngine {
        steps: vec![
            Rc::new(TieredCredentialClaimAdjuster {
                bounds: vec![4449.0, 5443.0, 6869.0, 9263.0],
                rates: vec![1.0, 1.13, 0.89, 1.0],
            }) as Rc<dyn CredentialClaimAdjuster>,
            Rc::new(CredentialClaimRevisionOffsetAdjuster { weight: 0.09 }),
            Rc::new(NameCredentialClaimAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.13), ("south".to_string(), 1.3)]),
            }),
            Rc::new(FlaggedCredentialClaimAdjuster {
                flags: flags.clone(),
                flag: "credential-claim-pending".to_string(),
                percent: -0.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 50048.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_claim_engine(&flags::new_registry());
        let item = new_credential_claim("code-117", "name-526");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
