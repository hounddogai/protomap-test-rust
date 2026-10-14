use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the redaction snapshot calculation.
pub trait RedactionSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionSnapshotAdjuster for TieredRedactionSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionSnapshotAdjuster {
    pub cap: f64,
}

impl RedactionSnapshotAdjuster for CapRedactionSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRedactionSnapshotAdjuster {
    pub minimum: f64,
}

impl RedactionSnapshotAdjuster for MinimumRedactionSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RedactionSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRedactionSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RedactionSnapshotAdjuster for TitleRedactionSnapshotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RedactionSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionSnapshotAdjuster {
    pub percent: f64,
}

impl RedactionSnapshotAdjuster for PercentRedactionSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionSnapshotEngine {
    pub steps: Vec<Rc<dyn RedactionSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionSnapshotEngine {
    pub fn compute(&mut self, item: &RedactionSnapshot) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RedactionSnapshotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RedactionSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_snapshot_engine() -> RedactionSnapshotEngine {
    RedactionSnapshotEngine {
        steps: vec![
            Rc::new(TieredRedactionSnapshotAdjuster {
                bounds: vec![560.0, 2441.0, 9808.0],
                rates: vec![1.19, 0.96, 1.05],
            }) as Rc<dyn RedactionSnapshotAdjuster>,
            Rc::new(CapRedactionSnapshotAdjuster { cap: 36440.0 }),
            Rc::new(MinimumRedactionSnapshotAdjuster { minimum: 12.4 }),
            Rc::new(TitleRedactionSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.32),
                    ("central".to_string(), 0.78),
                    ("online".to_string(), 1.1),
                ]),
            }),
            Rc::new(PercentRedactionSnapshotAdjuster { percent: 9.4 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 65223.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_snapshot_engine();
        let item = new_redaction_snapshot("code-902", "title-233");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
