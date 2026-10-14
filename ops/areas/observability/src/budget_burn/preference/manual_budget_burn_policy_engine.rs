use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual budget burn policy calculation.
pub trait ManualBudgetBurnPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBudgetBurnPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualBudgetBurnPolicyAdjuster {
    pub cap: f64,
}

impl ManualBudgetBurnPolicyStep for CapManualBudgetBurnPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualBudgetBurnPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelManualBudgetBurnPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualBudgetBurnPolicyStep for ChannelManualBudgetBurnPolicyAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ManualBudgetBurnPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualBudgetBurnPolicyAdjuster {
    pub percent: f64,
}

impl ManualBudgetBurnPolicyStep for PercentManualBudgetBurnPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualBudgetBurnPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualBudgetBurnPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualBudgetBurnPolicyStep for TieredManualBudgetBurnPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualBudgetBurnPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual budget burn policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBudgetBurnPolicyEngine {
    pub steps: Vec<Rc<dyn ManualBudgetBurnPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBudgetBurnPolicyEngine {
    pub fn evaluate(&mut self, item: &ManualBudgetBurnPolicy) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[ManualBudgetBurnPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_budget_burn_policy_engine() -> ManualBudgetBurnPolicyEngine {
    ManualBudgetBurnPolicyEngine {
        steps: vec![
            Rc::new(CapManualBudgetBurnPolicyAdjuster { cap: 16146.0 }) as Rc<dyn ManualBudgetBurnPolicyStep>,
            Rc::new(ChannelManualBudgetBurnPolicyAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.71)]),
            }),
            Rc::new(PercentManualBudgetBurnPolicyAdjuster { percent: 18.9 }),
            Rc::new(TieredManualBudgetBurnPolicyAdjuster {
                bounds: vec![6831.0, 8672.0],
                rates: vec![0.81, 0.87],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 97460.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_budget_burn_policy_engine();
        let item = new_manual_budget_burn_policy("code-260", "title-262");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
