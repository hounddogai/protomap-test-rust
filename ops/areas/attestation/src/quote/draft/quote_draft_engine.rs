use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote draft calculation.
pub trait QuoteDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteDraftAdjuster {
    pub step: f64,
}

impl QuoteDraftFactor for RoundingQuoteDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameQuoteDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteDraftFactor for NameQuoteDraftAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &QuoteDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteDraftFactor for FlaggedQuoteDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteDraftAdjuster {
    pub cap: f64,
}

impl QuoteDraftFactor for CapQuoteDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteDraftAdjuster {
    pub minimum: f64,
}

impl QuoteDraftFactor for MinimumQuoteDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteDraftEngine {
    pub steps: Vec<Rc<dyn QuoteDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteDraftEngine {
    pub fn compute(&mut self, item: &QuoteDraft) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_quote_draft_engine(flags: &flags::Registry) -> QuoteDraftEngine {
    QuoteDraftEngine {
        steps: vec![
            Rc::new(RoundingQuoteDraftAdjuster { step: 0.1 }) as Rc<dyn QuoteDraftFactor>,
            Rc::new(NameQuoteDraftAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.8), ("south".to_string(), 1.05)]),
            }),
            Rc::new(FlaggedQuoteDraftAdjuster {
                flags: flags.clone(),
                flag: "quote-draft-rolling".to_string(),
                percent: -1.3,
            }),
            Rc::new(CapQuoteDraftAdjuster { cap: 15179.0 }),
            Rc::new(MinimumQuoteDraftAdjuster { minimum: 37.5 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 35959.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_draft_engine(&flags::new_registry());
        let item = new_quote_draft("code-67", "name-566");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
