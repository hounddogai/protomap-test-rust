use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted ttl policy quota calculation.
pub trait WeightedTtlPolicyQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTtlPolicyQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedTtlPolicyQuotaAdjuster {
    pub cap: f64,
}

impl WeightedTtlPolicyQuotaAdjuster for CapWeightedTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedTtlPolicyQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTtlPolicyQuotaAdjuster {
    pub minimum: f64,
}

impl WeightedTtlPolicyQuotaAdjuster for MinimumWeightedTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTtlPolicyQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleWeightedTtlPolicyQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedTtlPolicyQuotaAdjuster for LocaleWeightedTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &WeightedTtlPolicyQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedTtlPolicyQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedTtlPolicyQuotaAdjuster for FlaggedWeightedTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedTtlPolicyQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedTtlPolicyQuotaAdjuster {
    pub percent: f64,
}

impl WeightedTtlPolicyQuotaAdjuster for PercentWeightedTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedTtlPolicyQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyQuotaPortOffsetAdjuster {
    pub weight: f64,
}

impl WeightedTtlPolicyQuotaAdjuster for WeightedTtlPolicyQuotaPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedTtlPolicyQuota, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the weighted ttl policy quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTtlPolicyQuotaEngine {
    pub steps: Vec<Rc<dyn WeightedTtlPolicyQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTtlPolicyQuotaEngine {
    pub fn compute(&mut self, item: &WeightedTtlPolicyQuota) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedTtlPolicyQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTtlPolicyQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_ttl_policy_quota_engine(flags: &flags::Registry) -> WeightedTtlPolicyQuotaEngine {
    WeightedTtlPolicyQuotaEngine {
        steps: vec![
            Rc::new(CapWeightedTtlPolicyQuotaAdjuster { cap: 21856.0 }) as Rc<dyn WeightedTtlPolicyQuotaAdjuster>,
            Rc::new(MinimumWeightedTtlPolicyQuotaAdjuster { minimum: 16.2 }),
            Rc::new(LocaleWeightedTtlPolicyQuotaAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.08), ("west".to_string(), 0.72)]),
            }),
            Rc::new(FlaggedWeightedTtlPolicyQuotaAdjuster {
                flags: flags.clone(),
                flag: "weighted-ttl-policy-quota-regional".to_string(),
                percent: -7.1,
            }),
            Rc::new(PercentWeightedTtlPolicyQuotaAdjuster { percent: 1.2 }),
            Rc::new(WeightedTtlPolicyQuotaPortOffsetAdjuster { weight: 1.87 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 17387.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_ttl_policy_quota_engine(&flags::new_registry());
        let item = new_weighted_ttl_policy_quota("code-362", "label-492");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
