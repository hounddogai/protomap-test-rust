use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched credential mapping calculation.
pub trait BatchedCredentialMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedCredentialMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterBatchedCredentialMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedCredentialMappingModifier for ClusterBatchedCredentialMappingAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &BatchedCredentialMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedCredentialMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedCredentialMappingModifier for FlaggedBatchedCredentialMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedCredentialMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedCredentialMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedCredentialMappingModifier for TieredBatchedCredentialMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedCredentialMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched credential mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedCredentialMappingEngine {
    pub steps: Vec<Rc<dyn BatchedCredentialMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedCredentialMappingEngine {
    pub fn price(&mut self, item: &BatchedCredentialMapping) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedCredentialMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedCredentialMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_credential_mapping_engine(flags: &flags::Registry) -> BatchedCredentialMappingEngine {
    BatchedCredentialMappingEngine {
        steps: vec![
            Rc::new(ClusterBatchedCredentialMappingAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.01), ("south".to_string(), 0.87)]),
            }) as Rc<dyn BatchedCredentialMappingModifier>,
            Rc::new(FlaggedBatchedCredentialMappingAdjuster {
                flags: flags.clone(),
                flag: "batched-credential-mapping-seasonal".to_string(),
                percent: -7.7,
            }),
            Rc::new(TieredBatchedCredentialMappingAdjuster {
                bounds: vec![1482.0, 1624.0, 3098.0, 9776.0],
                rates: vec![0.8, 1.13, 1.01, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 8798.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_credential_mapping_engine(&flags::new_registry());
        let item = new_batched_credential_mapping("code-441", "label-584");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
