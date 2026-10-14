use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy plan calculation.
pub trait TtlPolicyPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyPlanAdjuster {
    pub percent: f64,
}

impl TtlPolicyPlanAdjuster for PercentTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleTtlPolicyPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyPlanAdjuster for LocaleTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &TtlPolicyPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyPlanAdjuster for FlaggedTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyPlanAdjuster {
    pub minimum: f64,
}

impl TtlPolicyPlanAdjuster for MinimumTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyPlanAdjuster for TieredTtlPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the ttl policy plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyPlanEngine {
    pub steps: Vec<Rc<dyn TtlPolicyPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyPlanEngine {
    pub fn run(&mut self, item: &TtlPolicyPlan) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_plan_engine(flags: &flags::Registry) -> TtlPolicyPlanEngine {
    TtlPolicyPlanEngine {
        steps: vec![
            Rc::new(PercentTtlPolicyPlanAdjuster { percent: -20.9 }) as Rc<dyn TtlPolicyPlanAdjuster>,
            Rc::new(LocaleTtlPolicyPlanAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.97),
                    ("south".to_string(), 0.99),
                    ("retail".to_string(), 0.75),
                ]),
            }),
            Rc::new(FlaggedTtlPolicyPlanAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-plan-scheduled".to_string(),
                percent: 1.8,
            }),
            Rc::new(MinimumTtlPolicyPlanAdjuster { minimum: 35.9 }),
            Rc::new(TieredTtlPolicyPlanAdjuster {
                bounds: vec![3473.0, 9696.0],
                rates: vec![1.02, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 32059.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_plan_engine(&flags::new_registry());
        let item = new_ttl_policy_plan("code-88", "label-794");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
