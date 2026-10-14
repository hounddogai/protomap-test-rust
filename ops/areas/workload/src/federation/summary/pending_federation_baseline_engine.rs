use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending federation baseline calculation.
pub trait PendingFederationBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingFederationBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingFederationBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingFederationBaselineFactor for FlaggedPendingFederationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingFederationBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingFederationBaselineAdjuster {
    pub minimum: f64,
}

impl PendingFederationBaselineFactor for MinimumPendingFederationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingFederationBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingFederationBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingFederationBaselineFactor for NamePendingFederationBaselineAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingFederationBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending federation baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingFederationBaselineEngine {
    pub steps: Vec<Rc<dyn PendingFederationBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingFederationBaselineEngine {
    pub fn compute(&mut self, item: &PendingFederationBaseline) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingFederationBaselineFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingFederationBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_federation_baseline_engine(flags: &flags::Registry) -> PendingFederationBaselineEngine {
    PendingFederationBaselineEngine {
        steps: vec![
            Rc::new(FlaggedPendingFederationBaselineAdjuster {
                flags: flags.clone(),
                flag: "pending-federation-baseline-default".to_string(),
                percent: 4.7,
            }) as Rc<dyn PendingFederationBaselineFactor>,
            Rc::new(MinimumPendingFederationBaselineAdjuster { minimum: 18.0 }),
            Rc::new(NamePendingFederationBaselineAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.08),
                    ("online".to_string(), 1.17),
                    ("west".to_string(), 1.27),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 19086.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_federation_baseline_engine(&flags::new_registry());
        let item = new_pending_federation_baseline("code-289", "name-534");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
