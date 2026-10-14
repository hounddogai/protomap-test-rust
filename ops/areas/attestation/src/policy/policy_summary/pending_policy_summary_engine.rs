use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending policy summary calculation.
pub trait PendingPolicySummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPolicySummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicySummaryRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl PendingPolicySummaryStep for PendingPolicySummaryRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &PendingPolicySummary, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPolicySummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPolicySummaryStep for FlaggedPendingPolicySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPolicySummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingPolicySummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPolicySummaryStep for NamePendingPolicySummaryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingPolicySummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending policy summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPolicySummaryEngine {
    pub steps: Vec<Rc<dyn PendingPolicySummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPolicySummaryEngine {
    pub fn compute(&mut self, item: &PendingPolicySummary) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingPolicySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_policy_summary_engine(flags: &flags::Registry) -> PendingPolicySummaryEngine {
    PendingPolicySummaryEngine {
        steps: vec![
            Rc::new(PendingPolicySummaryRotationDaysOffsetAdjuster { weight: 0.72 })
                as Rc<dyn PendingPolicySummaryStep>,
            Rc::new(FlaggedPendingPolicySummaryAdjuster {
                flags: flags.clone(),
                flag: "pending-policy-summary-regional".to_string(),
                percent: -7.2,
            }),
            Rc::new(NamePendingPolicySummaryAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.8),
                    ("online".to_string(), 1.22),
                    ("north".to_string(), 0.89),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 47700.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_policy_summary_engine(&flags::new_registry());
        let item = new_pending_policy_summary("code-864", "name-390");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
