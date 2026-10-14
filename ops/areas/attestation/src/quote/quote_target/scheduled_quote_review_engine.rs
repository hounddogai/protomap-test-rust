use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled quote review calculation.
pub trait ScheduledQuoteReviewAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledQuoteReview, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledQuoteReviewAdjuster {
    pub percent: f64,
}

impl ScheduledQuoteReviewAdjuster for PercentScheduledQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledQuoteReview, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledQuoteReviewAdjuster {
    pub step: f64,
}

impl ScheduledQuoteReviewAdjuster for RoundingScheduledQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledQuoteReview, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryScheduledQuoteReviewAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledQuoteReviewAdjuster for CountryScheduledQuoteReviewAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ScheduledQuoteReview, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledQuoteReviewAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledQuoteReviewAdjuster for FlaggedScheduledQuoteReviewAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledQuoteReview, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled quote review steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledQuoteReviewEngine {
    pub steps: Vec<Rc<dyn ScheduledQuoteReviewAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledQuoteReviewEngine {
    pub fn run(&mut self, item: &ScheduledQuoteReview) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledQuoteReview]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_quote_review_engine(flags: &flags::Registry) -> ScheduledQuoteReviewEngine {
    ScheduledQuoteReviewEngine {
        steps: vec![
            Rc::new(PercentScheduledQuoteReviewAdjuster { percent: -12.4 }) as Rc<dyn ScheduledQuoteReviewAdjuster>,
            Rc::new(RoundingScheduledQuoteReviewAdjuster { step: 5.0 }),
            Rc::new(CountryScheduledQuoteReviewAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.78), ("south".to_string(), 1.29)]),
            }),
            Rc::new(FlaggedScheduledQuoteReviewAdjuster {
                flags: flags.clone(),
                flag: "scheduled-quote-review-manual".to_string(),
                percent: 9.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 44858.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_quote_review_engine(&flags::new_registry());
        let item = new_scheduled_quote_review("code-605", "label-588");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
