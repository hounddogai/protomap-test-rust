use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log policy calculation.
pub trait FlowLogPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogPolicyModifier for FlaggedFlowLogPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFlowLogPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogPolicyModifier for ClusterFlowLogPolicyAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FlowLogPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogPolicyModifier for TieredFlowLogPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogPolicyAdjuster {
    pub percent: f64,
}

impl FlowLogPolicyModifier for PercentFlowLogPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the flow log policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogPolicyEngine {
    pub steps: Vec<Rc<dyn FlowLogPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogPolicyEngine {
    pub fn evaluate(&mut self, item: &FlowLogPolicy) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_flow_log_policy_engine(flags: &flags::Registry) -> FlowLogPolicyEngine {
    FlowLogPolicyEngine {
        steps: vec![
            Rc::new(FlaggedFlowLogPolicyAdjuster {
                flags: flags.clone(),
                flag: "flow-log-policy-global".to_string(),
                percent: -6.2,
            }) as Rc<dyn FlowLogPolicyModifier>,
            Rc::new(ClusterFlowLogPolicyAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.03),
                    ("online".to_string(), 0.75),
                    ("east".to_string(), 0.85),
                ]),
            }),
            Rc::new(TieredFlowLogPolicyAdjuster {
                bounds: vec![5601.0, 5803.0],
                rates: vec![1.0, 1.03],
            }),
            Rc::new(PercentFlowLogPolicyAdjuster { percent: 19.6 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 10607.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_policy_engine(&flags::new_registry());
        let item = new_flow_log_policy("code-832", "title-967");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
