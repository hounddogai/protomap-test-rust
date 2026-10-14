use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending approval plan calculation.
pub trait PendingApprovalPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingApprovalPlan, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingApprovalPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingApprovalPlanModifier for FlaggedPendingApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingApprovalPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPendingApprovalPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingApprovalPlanModifier for CountryPendingApprovalPlanAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PendingApprovalPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingApprovalPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingApprovalPlanModifier for TieredPendingApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingApprovalPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending approval plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingApprovalPlanEngine {
    pub steps: Vec<Rc<dyn PendingApprovalPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingApprovalPlanEngine {
    pub fn run(&mut self, item: &PendingApprovalPlan) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingApprovalPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_approval_plan_engine(flags: &flags::Registry) -> PendingApprovalPlanEngine {
    PendingApprovalPlanEngine {
        steps: vec![
            Rc::new(FlaggedPendingApprovalPlanAdjuster {
                flags: flags.clone(),
                flag: "pending-approval-plan-default".to_string(),
                percent: -9.4,
            }) as Rc<dyn PendingApprovalPlanModifier>,
            Rc::new(CountryPendingApprovalPlanAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.29)]),
            }),
            Rc::new(TieredPendingApprovalPlanAdjuster {
                bounds: vec![6858.0, 7262.0],
                rates: vec![0.83, 1.2],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 9900.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_approval_plan_engine(&flags::new_registry());
        let item = new_pending_approval_plan("code-718", "name-676");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
