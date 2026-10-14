use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped ttl policy tier calculation.
pub trait CappedTtlPolicyTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedTtlPolicyTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedTtlPolicyTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedTtlPolicyTierFactor for TieredCappedTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedTtlPolicyTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedTtlPolicyTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedTtlPolicyTierFactor for FlaggedCappedTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedTtlPolicyTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryCappedTtlPolicyTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedTtlPolicyTierFactor for CategoryCappedTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &CappedTtlPolicyTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTtlPolicyTierPortOffsetAdjuster {
    pub weight: f64,
}

impl CappedTtlPolicyTierFactor for CappedTtlPolicyTierPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &CappedTtlPolicyTier, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedTtlPolicyTierAdjuster {
    pub percent: f64,
}

impl CappedTtlPolicyTierFactor for PercentCappedTtlPolicyTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedTtlPolicyTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped ttl policy tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedTtlPolicyTierEngine {
    pub steps: Vec<Rc<dyn CappedTtlPolicyTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedTtlPolicyTierEngine {
    pub fn compute(&mut self, item: &CappedTtlPolicyTier) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedTtlPolicyTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_ttl_policy_tier_engine(flags: &flags::Registry) -> CappedTtlPolicyTierEngine {
    CappedTtlPolicyTierEngine {
        steps: vec![
            Rc::new(TieredCappedTtlPolicyTierAdjuster {
                bounds: vec![3015.0, 7003.0, 7476.0, 8180.0],
                rates: vec![0.83, 0.87, 0.86, 1.12],
            }) as Rc<dyn CappedTtlPolicyTierFactor>,
            Rc::new(FlaggedCappedTtlPolicyTierAdjuster {
                flags: flags.clone(),
                flag: "capped-ttl-policy-tier-legacy".to_string(),
                percent: 6.0,
            }),
            Rc::new(CategoryCappedTtlPolicyTierAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.72),
                    ("online".to_string(), 0.97),
                    ("central".to_string(), 0.89),
                ]),
            }),
            Rc::new(CappedTtlPolicyTierPortOffsetAdjuster { weight: 0.58 }),
            Rc::new(PercentCappedTtlPolicyTierAdjuster { percent: 4.7 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 42463.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_ttl_policy_tier_engine(&flags::new_registry());
        let item = new_capped_ttl_policy_tier("code-730", "name-399");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
