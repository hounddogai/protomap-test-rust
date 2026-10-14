use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy mapping calculation.
pub trait TtlPolicyMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMappingAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyMappingModifier for TtlPolicyMappingAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyMapping, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyMappingModifier for TieredTtlPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyMappingAdjuster {
    pub step: f64,
}

impl TtlPolicyMappingModifier for RoundingTtlPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyMappingModifier for FlaggedTtlPolicyMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTtlPolicyMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyMappingModifier for ClusterTtlPolicyMappingAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &TtlPolicyMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the ttl policy mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyMappingEngine {
    pub steps: Vec<Rc<dyn TtlPolicyMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyMappingEngine {
    pub fn evaluate(&mut self, item: &TtlPolicyMapping) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyMappingModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_ttl_policy_mapping_engine(flags: &flags::Registry) -> TtlPolicyMappingEngine {
    TtlPolicyMappingEngine {
        steps: vec![
            Rc::new(TtlPolicyMappingAttemptsOffsetAdjuster { weight: 1.54 }) as Rc<dyn TtlPolicyMappingModifier>,
            Rc::new(TieredTtlPolicyMappingAdjuster {
                bounds: vec![2500.0, 8690.0, 9986.0],
                rates: vec![0.85, 0.85, 0.83],
            }),
            Rc::new(RoundingTtlPolicyMappingAdjuster { step: 0.1 }),
            Rc::new(FlaggedTtlPolicyMappingAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-mapping-scheduled".to_string(),
                percent: -5.3,
            }),
            Rc::new(ClusterTtlPolicyMappingAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.02),
                    ("retail".to_string(), 0.83),
                    ("north".to_string(), 1.28),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 92985.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_mapping_engine(&flags::new_registry());
        let item = new_ttl_policy_mapping("code-443", "name-731");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
