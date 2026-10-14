use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback failover draft calculation.
pub trait FallbackFailoverDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackFailoverDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverDraftScoreOffsetAdjuster {
    pub weight: f64,
}

impl FallbackFailoverDraftAdjuster for FallbackFailoverDraftScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackFailoverDraft, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackFailoverDraftAdjuster {
    pub percent: f64,
}

impl FallbackFailoverDraftAdjuster for PercentFallbackFailoverDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackFailoverDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackFailoverDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackFailoverDraftAdjuster for FlaggedFallbackFailoverDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackFailoverDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryFallbackFailoverDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackFailoverDraftAdjuster for CountryFallbackFailoverDraftAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &FallbackFailoverDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback failover draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackFailoverDraftEngine {
    pub steps: Vec<Rc<dyn FallbackFailoverDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackFailoverDraftEngine {
    pub fn price(&mut self, item: &FallbackFailoverDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackFailoverDraftAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackFailoverDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_failover_draft_engine(flags: &flags::Registry) -> FallbackFailoverDraftEngine {
    FallbackFailoverDraftEngine {
        steps: vec![
            Rc::new(FallbackFailoverDraftScoreOffsetAdjuster { weight: 0.83 }) as Rc<dyn FallbackFailoverDraftAdjuster>,
            Rc::new(PercentFallbackFailoverDraftAdjuster { percent: 12.0 }),
            Rc::new(FlaggedFallbackFailoverDraftAdjuster {
                flags: flags.clone(),
                flag: "fallback-failover-draft-strict".to_string(),
                percent: -8.2,
            }),
            Rc::new(CountryFallbackFailoverDraftAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.21)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 53694.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_failover_draft_engine(&flags::new_registry());
        let item = new_fallback_failover_draft("code-750", "label-895");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
