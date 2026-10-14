use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the quote bundle calculation.
pub trait QuoteBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteBundleAdjuster {
    pub cap: f64,
}

impl QuoteBundleModifier for CapQuoteBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteBundleModifier for FlaggedQuoteBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteBundleAdjuster {
    pub minimum: f64,
}

impl QuoteBundleModifier for MinimumQuoteBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteBundleEngine {
    pub steps: Vec<Rc<dyn QuoteBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteBundleEngine {
    pub fn run(&mut self, item: &QuoteBundle) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_quote_bundle_engine(flags: &flags::Registry) -> QuoteBundleEngine {
    QuoteBundleEngine {
        steps: vec![
            Rc::new(CapQuoteBundleAdjuster { cap: 22085.0 }) as Rc<dyn QuoteBundleModifier>,
            Rc::new(FlaggedQuoteBundleAdjuster {
                flags: flags.clone(),
                flag: "quote-bundle-primary".to_string(),
                percent: 3.3,
            }),
            Rc::new(MinimumQuoteBundleAdjuster { minimum: 9.3 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 29921.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_bundle_engine(&flags::new_registry());
        let item = new_quote_bundle("code-209", "title-491");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
