use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback redaction summary calculation.
pub trait FallbackRedactionSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRedactionSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRedactionSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRedactionSummaryStep for FlaggedFallbackRedactionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRedactionSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackRedactionSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRedactionSummaryStep for TitleFallbackRedactionSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackRedactionSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRedactionSummaryAdjuster {
    pub cap: f64,
}

impl FallbackRedactionSummaryStep for CapFallbackRedactionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRedactionSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRedactionSummaryAdjuster {
    pub minimum: f64,
}

impl FallbackRedactionSummaryStep for MinimumFallbackRedactionSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRedactionSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback redaction summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRedactionSummaryEngine {
    pub steps: Vec<Rc<dyn FallbackRedactionSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRedactionSummaryEngine {
    pub fn run(&mut self, item: &FallbackRedactionSummary) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackRedactionSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRedactionSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_redaction_summary_engine(flags: &flags::Registry) -> FallbackRedactionSummaryEngine {
    FallbackRedactionSummaryEngine {
        steps: vec![
            Rc::new(FlaggedFallbackRedactionSummaryAdjuster {
                flags: flags.clone(),
                flag: "fallback-redaction-summary-batched".to_string(),
                percent: -6.2,
            }) as Rc<dyn FallbackRedactionSummaryStep>,
            Rc::new(TitleFallbackRedactionSummaryAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.06),
                    ("online".to_string(), 1.2),
                    ("east".to_string(), 0.85),
                ]),
            }),
            Rc::new(CapFallbackRedactionSummaryAdjuster { cap: 4671.0 }),
            Rc::new(MinimumFallbackRedactionSummaryAdjuster { minimum: 35.1 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 83055.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_redaction_summary_engine(&flags::new_registry());
        let item = new_fallback_redaction_summary("code-776", "title-330");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
