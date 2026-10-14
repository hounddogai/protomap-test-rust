use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the grant draft calculation.
pub trait GrantDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantDraftAdjuster {
    pub minimum: f64,
}

impl GrantDraftModifier for MinimumGrantDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantDraftModifier for TieredGrantDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterGrantDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantDraftModifier for ClusterGrantDraftAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &GrantDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the grant draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantDraftEngine {
    pub steps: Vec<Rc<dyn GrantDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantDraftEngine {
    pub fn price(&mut self, item: &GrantDraft) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_draft_engine() -> GrantDraftEngine {
    GrantDraftEngine {
        steps: vec![
            Rc::new(MinimumGrantDraftAdjuster { minimum: 3.4 }) as Rc<dyn GrantDraftModifier>,
            Rc::new(TieredGrantDraftAdjuster {
                bounds: vec![993.0, 1242.0, 2962.0, 5570.0],
                rates: vec![1.11, 1.1, 1.0, 0.84],
            }),
            Rc::new(ClusterGrantDraftAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.7), ("west".to_string(), 0.97)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 46766.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_draft_engine();
        let item = new_grant_draft("code-744", "name-299");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
