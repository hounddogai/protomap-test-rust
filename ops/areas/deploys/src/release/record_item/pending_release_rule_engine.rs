use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending release rule calculation.
pub trait PendingReleaseRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingReleaseRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingReleaseRuleAdjuster {
    pub percent: f64,
}

impl PendingReleaseRuleAdjuster for PercentPendingReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingReleaseRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingReleaseRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingReleaseRuleAdjuster for FlaggedPendingReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingReleaseRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingReleaseRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingReleaseRuleAdjuster for TieredPendingReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingReleaseRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingReleaseRuleAdjuster {
    pub cap: f64,
}

impl PendingReleaseRuleAdjuster for CapPendingReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingReleaseRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPendingReleaseRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingReleaseRuleAdjuster for LabelPendingReleaseRuleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PendingReleaseRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending release rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingReleaseRuleEngine {
    pub steps: Vec<Rc<dyn PendingReleaseRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingReleaseRuleEngine {
    pub fn run(&mut self, item: &PendingReleaseRule) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingReleaseRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingReleaseRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_release_rule_engine(flags: &flags::Registry) -> PendingReleaseRuleEngine {
    PendingReleaseRuleEngine {
        steps: vec![
            Rc::new(PercentPendingReleaseRuleAdjuster { percent: 6.0 }) as Rc<dyn PendingReleaseRuleAdjuster>,
            Rc::new(FlaggedPendingReleaseRuleAdjuster {
                flags: flags.clone(),
                flag: "pending-release-rule-manual".to_string(),
                percent: -2.6,
            }),
            Rc::new(TieredPendingReleaseRuleAdjuster {
                bounds: vec![700.0, 955.0, 1765.0, 6834.0],
                rates: vec![1.09, 1.06, 1.18, 0.83],
            }),
            Rc::new(CapPendingReleaseRuleAdjuster { cap: 49641.0 }),
            Rc::new(LabelPendingReleaseRuleAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.95),
                    ("east".to_string(), 1.33),
                    ("central".to_string(), 0.73),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 86395.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_release_rule_engine(&flags::new_registry());
        let item = new_pending_release_rule("code-319", "label-395");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
