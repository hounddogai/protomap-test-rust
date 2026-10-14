use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped budget burn audit calculation.
pub trait CappedBudgetBurnAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedBudgetBurnAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnAuditTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl CappedBudgetBurnAuditModifier for CappedBudgetBurnAuditTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &CappedBudgetBurnAudit, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleCappedBudgetBurnAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedBudgetBurnAuditModifier for LocaleCappedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &CappedBudgetBurnAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedBudgetBurnAuditAdjuster {
    pub step: f64,
}

impl CappedBudgetBurnAuditModifier for RoundingCappedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedBudgetBurnAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedBudgetBurnAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedBudgetBurnAuditModifier for FlaggedCappedBudgetBurnAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedBudgetBurnAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped budget burn audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnAuditEngine {
    pub steps: Vec<Rc<dyn CappedBudgetBurnAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedBudgetBurnAuditEngine {
    pub fn compute(&mut self, item: &CappedBudgetBurnAudit) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[CappedBudgetBurnAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_budget_burn_audit_engine(flags: &flags::Registry) -> CappedBudgetBurnAuditEngine {
    CappedBudgetBurnAuditEngine {
        steps: vec![
            Rc::new(CappedBudgetBurnAuditTtlSecondsOffsetAdjuster { weight: 0.64 })
                as Rc<dyn CappedBudgetBurnAuditModifier>,
            Rc::new(LocaleCappedBudgetBurnAuditAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.82),
                    ("online".to_string(), 0.87),
                    ("north".to_string(), 1.33),
                ]),
            }),
            Rc::new(RoundingCappedBudgetBurnAuditAdjuster { step: 0.5 }),
            Rc::new(FlaggedCappedBudgetBurnAuditAdjuster {
                flags: flags.clone(),
                flag: "capped-budget-burn-audit-scheduled".to_string(),
                percent: 1.8,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 62602.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_budget_burn_audit_engine(&flags::new_registry());
        let item = new_capped_budget_burn_audit("code-127", "label-74");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
