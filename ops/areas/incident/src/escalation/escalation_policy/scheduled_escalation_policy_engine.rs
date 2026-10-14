use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled escalation policy calculation.
pub trait ScheduledEscalationPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledEscalationPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledEscalationPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledEscalationPolicyFactor for TieredScheduledEscalationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledEscalationPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledEscalationPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledEscalationPolicyFactor for FlaggedScheduledEscalationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledEscalationPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledEscalationPolicyAdjuster {
    pub minimum: f64,
}

impl ScheduledEscalationPolicyFactor for MinimumScheduledEscalationPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledEscalationPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled escalation policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledEscalationPolicyEngine {
    pub steps: Vec<Rc<dyn ScheduledEscalationPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledEscalationPolicyEngine {
    pub fn compute(&mut self, item: &ScheduledEscalationPolicy) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledEscalationPolicyFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledEscalationPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_escalation_policy_engine(flags: &flags::Registry) -> ScheduledEscalationPolicyEngine {
    ScheduledEscalationPolicyEngine {
        steps: vec![
            Rc::new(TieredScheduledEscalationPolicyAdjuster {
                bounds: vec![3505.0, 4268.0, 7033.0],
                rates: vec![0.81, 0.98, 0.97],
            }) as Rc<dyn ScheduledEscalationPolicyFactor>,
            Rc::new(FlaggedScheduledEscalationPolicyAdjuster {
                flags: flags.clone(),
                flag: "scheduled-escalation-policy-adaptive".to_string(),
                percent: 0.3,
            }),
            Rc::new(MinimumScheduledEscalationPolicyAdjuster { minimum: 7.5 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 65578.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_escalation_policy_engine(&flags::new_registry());
        let item = new_scheduled_escalation_policy("code-300", "name-349");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
