use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the quote schedule calculation.
pub trait QuoteScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteScheduleAdjuster {
    pub cap: f64,
}

impl QuoteScheduleStep for CapQuoteScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryQuoteScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteScheduleStep for CategoryQuoteScheduleAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &QuoteSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteScheduleStep for FlaggedQuoteScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteScheduleEngine {
    pub steps: Vec<Rc<dyn QuoteScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteScheduleEngine {
    pub fn run(&mut self, item: &QuoteSchedule) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteScheduleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_schedule_engine(flags: &flags::Registry) -> QuoteScheduleEngine {
    QuoteScheduleEngine {
        steps: vec![
            Rc::new(CapQuoteScheduleAdjuster { cap: 16744.0 }) as Rc<dyn QuoteScheduleStep>,
            Rc::new(CategoryQuoteScheduleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.05)]),
            }),
            Rc::new(FlaggedQuoteScheduleAdjuster {
                flags: flags.clone(),
                flag: "quote-schedule-fallback".to_string(),
                percent: 7.8,
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 13348.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_schedule_engine(&flags::new_registry());
        let item = new_quote_schedule("code-54", "name-605");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
