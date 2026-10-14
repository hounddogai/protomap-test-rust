use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the suspension draft calculation.
pub trait SuspensionDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionDraftAdjuster {
    pub minimum: f64,
}

impl SuspensionDraftModifier for MinimumSuspensionDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDraftCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionDraftModifier for SuspensionDraftCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionDraft, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSuspensionDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionDraftModifier for SourceSuspensionDraftAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SuspensionDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionDraftAdjuster {
    pub cap: f64,
}

impl SuspensionDraftModifier for CapSuspensionDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the suspension draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionDraftEngine {
    pub steps: Vec<Rc<dyn SuspensionDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionDraftEngine {
    pub fn evaluate(&mut self, item: &SuspensionDraft) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_draft_engine() -> SuspensionDraftEngine {
    SuspensionDraftEngine {
        steps: vec![
            Rc::new(MinimumSuspensionDraftAdjuster { minimum: 25.4 }) as Rc<dyn SuspensionDraftModifier>,
            Rc::new(SuspensionDraftCreatedAtOffsetAdjuster { weight: 1.53 }),
            Rc::new(SourceSuspensionDraftAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.32),
                    ("south".to_string(), 0.99),
                    ("east".to_string(), 1.01),
                ]),
            }),
            Rc::new(CapSuspensionDraftAdjuster { cap: 16244.0 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 34190.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_draft_engine();
        let item = new_suspension_draft("code-864", "title-794");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
