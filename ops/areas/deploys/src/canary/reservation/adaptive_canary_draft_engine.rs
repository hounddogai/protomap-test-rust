use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive canary draft calculation.
pub trait AdaptiveCanaryDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveCanaryDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveCanaryDraftAdjuster {
    pub cap: f64,
}

impl AdaptiveCanaryDraftStep for CapAdaptiveCanaryDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveCanaryDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveCanaryDraftAdjuster {
    pub minimum: f64,
}

impl AdaptiveCanaryDraftStep for MinimumAdaptiveCanaryDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveCanaryDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveCanaryDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveCanaryDraftStep for TieredAdaptiveCanaryDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveCanaryDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceAdaptiveCanaryDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveCanaryDraftStep for NamespaceAdaptiveCanaryDraftAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &AdaptiveCanaryDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive canary draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveCanaryDraftEngine {
    pub steps: Vec<Rc<dyn AdaptiveCanaryDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveCanaryDraftEngine {
    pub fn compute(&mut self, item: &AdaptiveCanaryDraft) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveCanaryDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveCanaryDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_canary_draft_engine() -> AdaptiveCanaryDraftEngine {
    AdaptiveCanaryDraftEngine {
        steps: vec![
            Rc::new(CapAdaptiveCanaryDraftAdjuster { cap: 40480.0 }) as Rc<dyn AdaptiveCanaryDraftStep>,
            Rc::new(MinimumAdaptiveCanaryDraftAdjuster { minimum: 47.8 }),
            Rc::new(TieredAdaptiveCanaryDraftAdjuster {
                bounds: vec![227.0, 2074.0, 2180.0],
                rates: vec![1.18, 0.9, 1.05],
            }),
            Rc::new(NamespaceAdaptiveCanaryDraftAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.07),
                    ("south".to_string(), 0.88),
                    ("online".to_string(), 1.2),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 60600.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_canary_draft_engine();
        let item = new_adaptive_canary_draft("code-320", "title-161");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
