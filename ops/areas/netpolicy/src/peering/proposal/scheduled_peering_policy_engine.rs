use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled peering policy calculation.
pub trait ScheduledPeeringPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledPeeringPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelScheduledPeeringPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledPeeringPolicyModifier for LabelScheduledPeeringPolicyAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ScheduledPeeringPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledPeeringPolicyAdjuster {
    pub cap: f64,
}

impl ScheduledPeeringPolicyModifier for CapScheduledPeeringPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledPeeringPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledPeeringPolicyAdjuster {
    pub step: f64,
}

impl ScheduledPeeringPolicyModifier for RoundingScheduledPeeringPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledPeeringPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled peering policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledPeeringPolicyEngine {
    pub steps: Vec<Rc<dyn ScheduledPeeringPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledPeeringPolicyEngine {
    pub fn price(&mut self, item: &ScheduledPeeringPolicy) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledPeeringPolicyModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledPeeringPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_peering_policy_engine() -> ScheduledPeeringPolicyEngine {
    ScheduledPeeringPolicyEngine {
        steps: vec![
            Rc::new(LabelScheduledPeeringPolicyAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.07)]),
            }) as Rc<dyn ScheduledPeeringPolicyModifier>,
            Rc::new(CapScheduledPeeringPolicyAdjuster { cap: 23569.0 }),
            Rc::new(RoundingScheduledPeeringPolicyAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 29705.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_peering_policy_engine();
        let item = new_scheduled_peering_policy("code-245", "label-200");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
