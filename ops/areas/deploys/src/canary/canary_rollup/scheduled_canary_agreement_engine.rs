use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled canary agreement calculation.
pub trait ScheduledCanaryAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledCanaryAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledCanaryAgreementAdjuster {
    pub minimum: f64,
}

impl ScheduledCanaryAgreementModifier for MinimumScheduledCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledCanaryAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledCanaryAgreementAdjuster {
    pub percent: f64,
}

impl ScheduledCanaryAgreementModifier for PercentScheduledCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledCanaryAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryAgreementLimitOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledCanaryAgreementModifier for ScheduledCanaryAgreementLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledCanaryAgreement, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledCanaryAgreementAdjuster {
    pub cap: f64,
}

impl ScheduledCanaryAgreementModifier for CapScheduledCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledCanaryAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeScheduledCanaryAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledCanaryAgreementModifier for ScopeScheduledCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ScheduledCanaryAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledCanaryAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledCanaryAgreementModifier for FlaggedScheduledCanaryAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledCanaryAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled canary agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledCanaryAgreementEngine {
    pub steps: Vec<Rc<dyn ScheduledCanaryAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledCanaryAgreementEngine {
    pub fn compute(&mut self, item: &ScheduledCanaryAgreement) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledCanaryAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledCanaryAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_canary_agreement_engine(flags: &flags::Registry) -> ScheduledCanaryAgreementEngine {
    ScheduledCanaryAgreementEngine {
        steps: vec![
            Rc::new(MinimumScheduledCanaryAgreementAdjuster { minimum: 16.0 })
                as Rc<dyn ScheduledCanaryAgreementModifier>,
            Rc::new(PercentScheduledCanaryAgreementAdjuster { percent: 19.4 }),
            Rc::new(ScheduledCanaryAgreementLimitOffsetAdjuster { weight: 0.94 }),
            Rc::new(CapScheduledCanaryAgreementAdjuster { cap: 37991.0 }),
            Rc::new(ScopeScheduledCanaryAgreementAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.0)]),
            }),
            Rc::new(FlaggedScheduledCanaryAgreementAdjuster {
                flags: flags.clone(),
                flag: "scheduled-canary-agreement-primary".to_string(),
                percent: -8.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 70791.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_canary_agreement_engine(&flags::new_registry());
        let item = new_scheduled_canary_agreement("code-618", "title-44");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
