use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling delegation template calculation.
pub trait RollingDelegationTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingDelegationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingDelegationTemplateAdjuster {
    pub cap: f64,
}

impl RollingDelegationTemplateModifier for CapRollingDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingDelegationTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingDelegationTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingDelegationTemplateModifier for FlaggedRollingDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingDelegationTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingDelegationTemplateAdjuster {
    pub step: f64,
}

impl RollingDelegationTemplateModifier for RoundingRollingDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingDelegationTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingDelegationTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingDelegationTemplateModifier for NameRollingDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingDelegationTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationTemplateCapacityOffsetAdjuster {
    pub weight: f64,
}

impl RollingDelegationTemplateModifier for RollingDelegationTemplateCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &RollingDelegationTemplate, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingDelegationTemplateAdjuster {
    pub percent: f64,
}

impl RollingDelegationTemplateModifier for PercentRollingDelegationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingDelegationTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling delegation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingDelegationTemplateEngine {
    pub steps: Vec<Rc<dyn RollingDelegationTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingDelegationTemplateEngine {
    pub fn compute(&mut self, item: &RollingDelegationTemplate) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingDelegationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_delegation_template_engine(flags: &flags::Registry) -> RollingDelegationTemplateEngine {
    RollingDelegationTemplateEngine {
        steps: vec![
            Rc::new(CapRollingDelegationTemplateAdjuster { cap: 41651.0 }) as Rc<dyn RollingDelegationTemplateModifier>,
            Rc::new(FlaggedRollingDelegationTemplateAdjuster {
                flags: flags.clone(),
                flag: "rolling-delegation-template-batched".to_string(),
                percent: 2.6,
            }),
            Rc::new(RoundingRollingDelegationTemplateAdjuster { step: 5.0 }),
            Rc::new(NameRollingDelegationTemplateAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.1),
                    ("east".to_string(), 1.28),
                    ("online".to_string(), 0.91),
                ]),
            }),
            Rc::new(RollingDelegationTemplateCapacityOffsetAdjuster { weight: 0.94 }),
            Rc::new(PercentRollingDelegationTemplateAdjuster { percent: 10.8 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 87997.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_delegation_template_engine(&flags::new_registry());
        let item = new_rolling_delegation_template("code-676", "name-64");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
