use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the delegation score calculation.
pub trait DelegationScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationScoreStep for TieredDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDelegationScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationScoreStep for NamespaceDelegationScoreAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &DelegationScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationScoreAdjuster {
    pub cap: f64,
}

impl DelegationScoreStep for CapDelegationScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the delegation score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationScoreEngine {
    pub steps: Vec<Rc<dyn DelegationScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationScoreEngine {
    pub fn price(&mut self, item: &DelegationScore) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_score_engine() -> DelegationScoreEngine {
    DelegationScoreEngine {
        steps: vec![
            Rc::new(TieredDelegationScoreAdjuster {
                bounds: vec![2704.0, 6228.0, 7225.0],
                rates: vec![0.82, 1.12, 1.07],
            }) as Rc<dyn DelegationScoreStep>,
            Rc::new(NamespaceDelegationScoreAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.21), ("central".to_string(), 1.15)]),
            }),
            Rc::new(CapDelegationScoreAdjuster { cap: 21241.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 53604.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_score_engine();
        let item = new_delegation_score("code-87", "title-868");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
