use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the policy summary calculation.
pub trait PolicySummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicySummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPolicySummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicySummaryModifier for ChannelPolicySummaryAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PolicySummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicySummaryAdjuster {
    pub percent: f64,
}

impl PolicySummaryModifier for PercentPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicySummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicySummaryAdjuster {
    pub cap: f64,
}

impl PolicySummaryModifier for CapPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicySummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicySummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicySummaryModifier for FlaggedPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicySummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicySummaryBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PolicySummaryModifier for PolicySummaryBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PolicySummary, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

/// Runs the policy summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicySummaryEngine {
    pub steps: Vec<Rc<dyn PolicySummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicySummaryEngine {
    pub fn compute(&mut self, item: &PolicySummary) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicySummaryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_policy_summary_engine(flags: &flags::Registry) -> PolicySummaryEngine {
    PolicySummaryEngine {
        steps: vec![
            Rc::new(ChannelPolicySummaryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.14)]),
            }) as Rc<dyn PolicySummaryModifier>,
            Rc::new(PercentPolicySummaryAdjuster { percent: -14.7 }),
            Rc::new(CapPolicySummaryAdjuster { cap: 24789.0 }),
            Rc::new(FlaggedPolicySummaryAdjuster {
                flags: flags.clone(),
                flag: "policy-summary-manual".to_string(),
                percent: -8.2,
            }),
            Rc::new(PolicySummaryBatchSizeOffsetAdjuster { weight: 0.71 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 63102.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_summary_engine(&flags::new_registry());
        let item = new_policy_summary("code-252", "title-590");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
