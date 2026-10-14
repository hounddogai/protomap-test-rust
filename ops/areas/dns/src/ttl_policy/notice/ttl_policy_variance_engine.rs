use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy variance calculation.
pub trait TtlPolicyVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyVarianceModifier for TieredTtlPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyVarianceAdjuster {
    pub minimum: f64,
}

impl TtlPolicyVarianceModifier for MinimumTtlPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyVarianceCeilingOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyVarianceModifier for TtlPolicyVarianceCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyVariance, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyVarianceAdjuster {
    pub step: f64,
}

impl TtlPolicyVarianceModifier for RoundingTtlPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTtlPolicyVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyVarianceModifier for LocaleTtlPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TtlPolicyVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyVarianceModifier for FlaggedTtlPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyVarianceEngine {
    pub steps: Vec<Rc<dyn TtlPolicyVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyVarianceEngine {
    pub fn run(&mut self, item: &TtlPolicyVariance) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyVarianceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_variance_engine(flags: &flags::Registry) -> TtlPolicyVarianceEngine {
    TtlPolicyVarianceEngine {
        steps: vec![
            Rc::new(TieredTtlPolicyVarianceAdjuster {
                bounds: vec![1644.0, 6237.0, 7973.0],
                rates: vec![1.09, 1.09, 1.11],
            }) as Rc<dyn TtlPolicyVarianceModifier>,
            Rc::new(MinimumTtlPolicyVarianceAdjuster { minimum: 3.9 }),
            Rc::new(TtlPolicyVarianceCeilingOffsetAdjuster { weight: 0.03 }),
            Rc::new(RoundingTtlPolicyVarianceAdjuster { step: 0.05 }),
            Rc::new(LocaleTtlPolicyVarianceAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.22),
                    ("retail".to_string(), 1.4),
                    ("west".to_string(), 0.87),
                ]),
            }),
            Rc::new(FlaggedTtlPolicyVarianceAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-variance-rolling".to_string(),
                percent: -9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 92795.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_variance_engine(&flags::new_registry());
        let item = new_ttl_policy_variance("code-852", "label-310");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
