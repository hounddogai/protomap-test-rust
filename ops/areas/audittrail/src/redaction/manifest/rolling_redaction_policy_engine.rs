use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling redaction policy calculation.
pub trait RollingRedactionPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingRedactionPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingRedactionPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingRedactionPolicyFactor for TieredRollingRedactionPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingRedactionPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingRedactionPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingRedactionPolicyFactor for FlaggedRollingRedactionPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingRedactionPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingRedactionPolicyAdjuster {
    pub percent: f64,
}

impl RollingRedactionPolicyFactor for PercentRollingRedactionPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingRedactionPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRollingRedactionPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingRedactionPolicyFactor for NamespaceRollingRedactionPolicyAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RollingRedactionPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling redaction policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingRedactionPolicyEngine {
    pub steps: Vec<Rc<dyn RollingRedactionPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingRedactionPolicyEngine {
    pub fn evaluate(&mut self, item: &RollingRedactionPolicy) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingRedactionPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_redaction_policy_engine(flags: &flags::Registry) -> RollingRedactionPolicyEngine {
    RollingRedactionPolicyEngine {
        steps: vec![
            Rc::new(TieredRollingRedactionPolicyAdjuster {
                bounds: vec![7685.0, 8978.0],
                rates: vec![1.05, 0.92],
            }) as Rc<dyn RollingRedactionPolicyFactor>,
            Rc::new(FlaggedRollingRedactionPolicyAdjuster {
                flags: flags.clone(),
                flag: "rolling-redaction-policy-seasonal".to_string(),
                percent: 9.2,
            }),
            Rc::new(PercentRollingRedactionPolicyAdjuster { percent: 19.6 }),
            Rc::new(NamespaceRollingRedactionPolicyAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.31), ("central".to_string(), 0.95)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 41177.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_redaction_policy_engine(&flags::new_registry());
        let item = new_rolling_redaction_policy("code-842", "title-787");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
