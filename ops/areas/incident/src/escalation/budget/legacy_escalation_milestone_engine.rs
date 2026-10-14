use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy escalation milestone calculation.
pub trait LegacyEscalationMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEscalationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyEscalationMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyEscalationMilestoneAdjuster for NameLegacyEscalationMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyEscalationMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEscalationMilestoneLimitOffsetAdjuster {
    pub weight: f64,
}

impl LegacyEscalationMilestoneAdjuster for LegacyEscalationMilestoneLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyEscalationMilestone, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyEscalationMilestoneAdjuster {
    pub minimum: f64,
}

impl LegacyEscalationMilestoneAdjuster for MinimumLegacyEscalationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyEscalationMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy escalation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEscalationMilestoneEngine {
    pub steps: Vec<Rc<dyn LegacyEscalationMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEscalationMilestoneEngine {
    pub fn price(&mut self, item: &LegacyEscalationMilestone) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyEscalationMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEscalationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_escalation_milestone_engine() -> LegacyEscalationMilestoneEngine {
    LegacyEscalationMilestoneEngine {
        steps: vec![
            Rc::new(NameLegacyEscalationMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.8),
                    ("online".to_string(), 0.88),
                    ("retail".to_string(), 1.31),
                ]),
            }) as Rc<dyn LegacyEscalationMilestoneAdjuster>,
            Rc::new(LegacyEscalationMilestoneLimitOffsetAdjuster { weight: 0.73 }),
            Rc::new(MinimumLegacyEscalationMilestoneAdjuster { minimum: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 59087.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_escalation_milestone_engine();
        let item = new_legacy_escalation_milestone("code-146", "name-465");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
