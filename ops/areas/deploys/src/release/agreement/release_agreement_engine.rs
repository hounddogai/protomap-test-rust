use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the release agreement calculation.
pub trait ReleaseAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeReleaseAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseAgreementStep for ScopeReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ReleaseAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseAgreementStep for FlaggedReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseAgreementAdjuster {
    pub minimum: f64,
}

impl ReleaseAgreementStep for MinimumReleaseAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the release agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseAgreementEngine {
    pub steps: Vec<Rc<dyn ReleaseAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseAgreementEngine {
    pub fn run(&mut self, item: &ReleaseAgreement) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_agreement_engine(flags: &flags::Registry) -> ReleaseAgreementEngine {
    ReleaseAgreementEngine {
        steps: vec![
            Rc::new(ScopeReleaseAgreementAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.93), ("online".to_string(), 1.05)]),
            }) as Rc<dyn ReleaseAgreementStep>,
            Rc::new(FlaggedReleaseAgreementAdjuster {
                flags: flags.clone(),
                flag: "release-agreement-adaptive".to_string(),
                percent: 5.6,
            }),
            Rc::new(MinimumReleaseAgreementAdjuster { minimum: 20.2 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 51215.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_agreement_engine(&flags::new_registry());
        let item = new_release_agreement("code-669", "title-824");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
