use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive credential exception calculation.
pub trait AdaptiveCredentialExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveCredentialException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveCredentialExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveCredentialExceptionFactor for TieredAdaptiveCredentialExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveCredentialException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveCredentialExceptionAdjuster {
    pub minimum: f64,
}

impl AdaptiveCredentialExceptionFactor for MinimumAdaptiveCredentialExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveCredentialException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialExceptionEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveCredentialExceptionFactor for AdaptiveCredentialExceptionEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveCredentialException, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryAdaptiveCredentialExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveCredentialExceptionFactor for CategoryAdaptiveCredentialExceptionAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &AdaptiveCredentialException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveCredentialExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveCredentialExceptionFactor for FlaggedAdaptiveCredentialExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveCredentialException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveCredentialExceptionAdjuster {
    pub cap: f64,
}

impl AdaptiveCredentialExceptionFactor for CapAdaptiveCredentialExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveCredentialException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive credential exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveCredentialExceptionEngine {
    pub steps: Vec<Rc<dyn AdaptiveCredentialExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveCredentialExceptionEngine {
    pub fn compute(&mut self, item: &AdaptiveCredentialException) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveCredentialExceptionFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveCredentialException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_credential_exception_engine(flags: &flags::Registry) -> AdaptiveCredentialExceptionEngine {
    AdaptiveCredentialExceptionEngine {
        steps: vec![
            Rc::new(TieredAdaptiveCredentialExceptionAdjuster {
                bounds: vec![2898.0, 6847.0],
                rates: vec![0.97, 1.05],
            }) as Rc<dyn AdaptiveCredentialExceptionFactor>,
            Rc::new(MinimumAdaptiveCredentialExceptionAdjuster { minimum: 14.7 }),
            Rc::new(AdaptiveCredentialExceptionEndsAtOffsetAdjuster { weight: 1.81 }),
            Rc::new(CategoryAdaptiveCredentialExceptionAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.71), ("west".to_string(), 1.16)]),
            }),
            Rc::new(FlaggedAdaptiveCredentialExceptionAdjuster {
                flags: flags.clone(),
                flag: "adaptive-credential-exception-rolling".to_string(),
                percent: 8.6,
            }),
            Rc::new(CapAdaptiveCredentialExceptionAdjuster { cap: 15357.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 20367.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_credential_exception_engine(&flags::new_registry());
        let item = new_adaptive_credential_exception("code-122", "label-44");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
