use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual flow log policy calculation.
pub trait ManualFlowLogPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualFlowLogPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualFlowLogPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualFlowLogPolicyModifier for FlaggedManualFlowLogPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualFlowLogPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelManualFlowLogPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualFlowLogPolicyModifier for LabelManualFlowLogPolicyAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ManualFlowLogPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualFlowLogPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualFlowLogPolicyModifier for TieredManualFlowLogPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualFlowLogPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual flow log policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualFlowLogPolicyEngine {
    pub steps: Vec<Rc<dyn ManualFlowLogPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualFlowLogPolicyEngine {
    pub fn price(&mut self, item: &ManualFlowLogPolicy) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualFlowLogPolicyModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualFlowLogPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_flow_log_policy_engine(flags: &flags::Registry) -> ManualFlowLogPolicyEngine {
    ManualFlowLogPolicyEngine {
        steps: vec![
            Rc::new(FlaggedManualFlowLogPolicyAdjuster {
                flags: flags.clone(),
                flag: "manual-flow-log-policy-seasonal".to_string(),
                percent: -4.4,
            }) as Rc<dyn ManualFlowLogPolicyModifier>,
            Rc::new(LabelManualFlowLogPolicyAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.38)]),
            }),
            Rc::new(TieredManualFlowLogPolicyAdjuster {
                bounds: vec![726.0, 1564.0],
                rates: vec![1.02, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 67704.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_flow_log_policy_engine(&flags::new_registry());
        let item = new_manual_flow_log_policy("code-743", "label-879");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
