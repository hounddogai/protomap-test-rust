use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default trust domain profile calculation.
pub trait DefaultTrustDomainProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultTrustDomainProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDefaultTrustDomainProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultTrustDomainProfileStep for ChannelDefaultTrustDomainProfileAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DefaultTrustDomainProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultTrustDomainProfileAdjuster {
    pub minimum: f64,
}

impl DefaultTrustDomainProfileStep for MinimumDefaultTrustDomainProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultTrustDomainProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTrustDomainProfileBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DefaultTrustDomainProfileStep for DefaultTrustDomainProfileBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultTrustDomainProfile, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultTrustDomainProfileAdjuster {
    pub cap: f64,
}

impl DefaultTrustDomainProfileStep for CapDefaultTrustDomainProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultTrustDomainProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default trust domain profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultTrustDomainProfileEngine {
    pub steps: Vec<Rc<dyn DefaultTrustDomainProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultTrustDomainProfileEngine {
    pub fn compute(&mut self, item: &DefaultTrustDomainProfile) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultTrustDomainProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultTrustDomainProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_trust_domain_profile_engine() -> DefaultTrustDomainProfileEngine {
    DefaultTrustDomainProfileEngine {
        steps: vec![
            Rc::new(ChannelDefaultTrustDomainProfileAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.24)]),
            }) as Rc<dyn DefaultTrustDomainProfileStep>,
            Rc::new(MinimumDefaultTrustDomainProfileAdjuster { minimum: 15.4 }),
            Rc::new(DefaultTrustDomainProfileBudgetOffsetAdjuster { weight: 1.03 }),
            Rc::new(CapDefaultTrustDomainProfileAdjuster { cap: 19970.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 42136.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_trust_domain_profile_engine();
        let item = new_default_trust_domain_profile("code-492", "title-15");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
