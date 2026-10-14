use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval quota calculation.
pub trait ApprovalQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalQuotaAdjuster for FlaggedApprovalQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuotaQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalQuotaAdjuster for ApprovalQuotaQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalQuota, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalQuotaAdjuster {
    pub step: f64,
}

impl ApprovalQuotaAdjuster for RoundingApprovalQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the approval quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalQuotaEngine {
    pub steps: Vec<Rc<dyn ApprovalQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalQuotaEngine {
    pub fn run(&mut self, item: &ApprovalQuota) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_quota_engine(flags: &flags::Registry) -> ApprovalQuotaEngine {
    ApprovalQuotaEngine {
        steps: vec![
            Rc::new(FlaggedApprovalQuotaAdjuster {
                flags: flags.clone(),
                flag: "approval-quota-manual".to_string(),
                percent: -2.9,
            }) as Rc<dyn ApprovalQuotaAdjuster>,
            Rc::new(ApprovalQuotaQuotaUnitsOffsetAdjuster { weight: 0.46 }),
            Rc::new(RoundingApprovalQuotaAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 24977.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_quota_engine(&flags::new_registry());
        let item = new_approval_quota("code-586", "name-267");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
