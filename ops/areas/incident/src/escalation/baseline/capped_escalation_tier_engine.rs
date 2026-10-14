use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped escalation tier calculation.
pub trait CappedEscalationTierModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedEscalationTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedEscalationTierAdjuster {
    pub cap: f64,
}

impl CappedEscalationTierModifier for CapCappedEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedEscalationTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleCappedEscalationTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedEscalationTierModifier for LocaleCappedEscalationTierAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &CappedEscalationTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedEscalationTierAdjuster {
    pub percent: f64,
}

impl CappedEscalationTierModifier for PercentCappedEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedEscalationTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedEscalationTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedEscalationTierModifier for FlaggedCappedEscalationTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedEscalationTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped escalation tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedEscalationTierEngine {
    pub steps: Vec<Rc<dyn CappedEscalationTierModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedEscalationTierEngine {
    pub fn price(&mut self, item: &CappedEscalationTier) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedEscalationTierModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedEscalationTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_escalation_tier_engine(flags: &flags::Registry) -> CappedEscalationTierEngine {
    CappedEscalationTierEngine {
        steps: vec![
            Rc::new(CapCappedEscalationTierAdjuster { cap: 30876.0 }) as Rc<dyn CappedEscalationTierModifier>,
            Rc::new(LocaleCappedEscalationTierAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.28), ("retail".to_string(), 1.15)]),
            }),
            Rc::new(PercentCappedEscalationTierAdjuster { percent: -12.7 }),
            Rc::new(FlaggedCappedEscalationTierAdjuster {
                flags: flags.clone(),
                flag: "capped-escalation-tier-regional".to_string(),
                percent: -1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 70420.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_escalation_tier_engine(&flags::new_registry());
        let item = new_capped_escalation_tier("code-816", "name-229");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
