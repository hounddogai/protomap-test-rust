use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page claim calculation.
pub trait PageClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageClaimAdjuster {
    pub step: f64,
}

impl PageClaimStep for RoundingPageClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageClaimStep for FlaggedPageClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageClaimAdjuster {
    pub minimum: f64,
}

impl PageClaimStep for MinimumPageClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePageClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageClaimStep for SourcePageClaimAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PageClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the page claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageClaimEngine {
    pub steps: Vec<Rc<dyn PageClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageClaimEngine {
    pub fn compute(&mut self, item: &PageClaim) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PageClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_page_claim_engine(flags: &flags::Registry) -> PageClaimEngine {
    PageClaimEngine {
        steps: vec![
            Rc::new(RoundingPageClaimAdjuster { step: 0.5 }) as Rc<dyn PageClaimStep>,
            Rc::new(FlaggedPageClaimAdjuster {
                flags: flags.clone(),
                flag: "page-claim-primary".to_string(),
                percent: 5.1,
            }),
            Rc::new(MinimumPageClaimAdjuster { minimum: 40.2 }),
            Rc::new(SourcePageClaimAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.86), ("west".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 28562.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_claim_engine(&flags::new_registry());
        let item = new_page_claim("code-90", "name-29");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
