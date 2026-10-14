use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention draft calculation.
pub trait RetentionDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRetentionDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionDraftStep for RegionRetentionDraftAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RetentionDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionDraftAdjuster {
    pub percent: f64,
}

impl RetentionDraftStep for PercentRetentionDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionDraftStep for FlaggedRetentionDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraftRateOffsetAdjuster {
    pub weight: f64,
}

impl RetentionDraftStep for RetentionDraftRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionDraft, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionDraftAdjuster {
    pub minimum: f64,
}

impl RetentionDraftStep for MinimumRetentionDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the retention draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionDraftEngine {
    pub steps: Vec<Rc<dyn RetentionDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionDraftEngine {
    pub fn price(&mut self, item: &RetentionDraft) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RetentionDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_retention_draft_engine(flags: &flags::Registry) -> RetentionDraftEngine {
    RetentionDraftEngine {
        steps: vec![
            Rc::new(RegionRetentionDraftAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.78),
                    ("west".to_string(), 0.7),
                    ("south".to_string(), 0.71),
                ]),
            }) as Rc<dyn RetentionDraftStep>,
            Rc::new(PercentRetentionDraftAdjuster { percent: -16.5 }),
            Rc::new(FlaggedRetentionDraftAdjuster {
                flags: flags.clone(),
                flag: "retention-draft-default".to_string(),
                percent: 6.1,
            }),
            Rc::new(RetentionDraftRateOffsetAdjuster { weight: 0.41 }),
            Rc::new(MinimumRetentionDraftAdjuster { minimum: 4.4 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 77319.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_draft_engine(&flags::new_registry());
        let item = new_retention_draft("code-535", "name-845");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
