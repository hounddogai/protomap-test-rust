use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy exception calculation.
pub trait TtlPolicyExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyExceptionModifier for FlaggedTtlPolicyExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyExceptionModifier for TieredTtlPolicyExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyExceptionRatioOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyExceptionModifier for TtlPolicyExceptionRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyException, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyExceptionAdjuster {
    pub minimum: f64,
}

impl TtlPolicyExceptionModifier for MinimumTtlPolicyExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the ttl policy exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyExceptionEngine {
    pub steps: Vec<Rc<dyn TtlPolicyExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyExceptionEngine {
    pub fn compute(&mut self, item: &TtlPolicyException) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_ttl_policy_exception_engine(flags: &flags::Registry) -> TtlPolicyExceptionEngine {
    TtlPolicyExceptionEngine {
        steps: vec![
            Rc::new(FlaggedTtlPolicyExceptionAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-exception-legacy".to_string(),
                percent: 0.7,
            }) as Rc<dyn TtlPolicyExceptionModifier>,
            Rc::new(TieredTtlPolicyExceptionAdjuster {
                bounds: vec![2855.0, 5104.0, 6178.0, 9194.0],
                rates: vec![0.97, 1.15, 1.05, 1.07],
            }),
            Rc::new(TtlPolicyExceptionRatioOffsetAdjuster { weight: 1.54 }),
            Rc::new(MinimumTtlPolicyExceptionAdjuster { minimum: 4.2 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 25271.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_exception_engine(&flags::new_registry());
        let item = new_ttl_policy_exception("code-191", "name-964");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
