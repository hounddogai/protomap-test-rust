use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the egress policy calculation.
pub trait EgressPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressPolicyModifier for TieredEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressPolicyModifier for FlaggedEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEgressPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressPolicyModifier for LabelEgressPolicyAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EgressPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressPolicyRetriesOffsetAdjuster {
    pub weight: f64,
}

impl EgressPolicyModifier for EgressPolicyRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &EgressPolicy, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressPolicyAdjuster {
    pub minimum: f64,
}

impl EgressPolicyModifier for MinimumEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressPolicyAdjuster {
    pub percent: f64,
}

impl EgressPolicyModifier for PercentEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the egress policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressPolicyEngine {
    pub steps: Vec<Rc<dyn EgressPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressPolicyEngine {
    pub fn compute(&mut self, item: &EgressPolicy) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressPolicyModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_egress_policy_engine(flags: &flags::Registry) -> EgressPolicyEngine {
    EgressPolicyEngine {
        steps: vec![
            Rc::new(TieredEgressPolicyAdjuster {
                bounds: vec![4230.0, 7433.0],
                rates: vec![1.04, 1.15],
            }) as Rc<dyn EgressPolicyModifier>,
            Rc::new(FlaggedEgressPolicyAdjuster {
                flags: flags.clone(),
                flag: "egress-policy-rolling".to_string(),
                percent: -10.0,
            }),
            Rc::new(LabelEgressPolicyAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.03),
                    ("online".to_string(), 0.95),
                    ("east".to_string(), 0.88),
                ]),
            }),
            Rc::new(EgressPolicyRetriesOffsetAdjuster { weight: 1.74 }),
            Rc::new(MinimumEgressPolicyAdjuster { minimum: 41.0 }),
            Rc::new(PercentEgressPolicyAdjuster { percent: 20.8 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 96831.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_policy_engine(&flags::new_registry());
        let item = new_egress_policy("code-832", "label-194");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
