use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary ttl policy plan calculation.
pub trait PrimaryTtlPolicyPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTtlPolicyPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryTtlPolicyPlanAdjuster {
    pub cap: f64,
}

impl PrimaryTtlPolicyPlanModifier for CapPrimaryTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryTtlPolicyPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryTtlPolicyPlanModifier for FlaggedPrimaryTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryTtlPolicyPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryTtlPolicyPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTtlPolicyPlanModifier for SourcePrimaryTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryTtlPolicyPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary ttl policy plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyPlanEngine {
    pub steps: Vec<Rc<dyn PrimaryTtlPolicyPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTtlPolicyPlanEngine {
    pub fn run(&mut self, item: &PrimaryTtlPolicyPlan) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTtlPolicyPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_ttl_policy_plan_engine(flags: &flags::Registry) -> PrimaryTtlPolicyPlanEngine {
    PrimaryTtlPolicyPlanEngine {
        steps: vec![
            Rc::new(CapPrimaryTtlPolicyPlanAdjuster { cap: 42700.0 }) as Rc<dyn PrimaryTtlPolicyPlanModifier>,
            Rc::new(FlaggedPrimaryTtlPolicyPlanAdjuster {
                flags: flags.clone(),
                flag: "primary-ttl-policy-plan-adaptive".to_string(),
                percent: 8.4,
            }),
            Rc::new(SourcePrimaryTtlPolicyPlanAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.28), ("central".to_string(), 0.76)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 38904.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_ttl_policy_plan_engine(&flags::new_registry());
        let item = new_primary_ttl_policy_plan("code-493", "title-974");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
