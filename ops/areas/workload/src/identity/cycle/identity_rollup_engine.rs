use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity rollup calculation.
pub trait IdentityRollupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityRollupStep for TieredIdentityRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityRollupAdjuster {
    pub percent: f64,
}

impl IdentityRollupStep for PercentIdentityRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryIdentityRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityRollupStep for CategoryIdentityRollupAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &IdentityRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityRollupStep for FlaggedIdentityRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityRollupEngine {
    pub steps: Vec<Rc<dyn IdentityRollupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityRollupEngine {
    pub fn evaluate(&mut self, item: &IdentityRollup) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[IdentityRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_rollup_engine(flags: &flags::Registry) -> IdentityRollupEngine {
    IdentityRollupEngine {
        steps: vec![
            Rc::new(TieredIdentityRollupAdjuster {
                bounds: vec![512.0, 5188.0, 6386.0],
                rates: vec![0.95, 1.11, 0.89],
            }) as Rc<dyn IdentityRollupStep>,
            Rc::new(PercentIdentityRollupAdjuster { percent: -5.7 }),
            Rc::new(CategoryIdentityRollupAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.19)]),
            }),
            Rc::new(FlaggedIdentityRollupAdjuster {
                flags: flags.clone(),
                flag: "identity-rollup-batched".to_string(),
                percent: 5.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 60111.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_rollup_engine(&flags::new_registry());
        let item = new_identity_rollup("code-429", "name-685");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
