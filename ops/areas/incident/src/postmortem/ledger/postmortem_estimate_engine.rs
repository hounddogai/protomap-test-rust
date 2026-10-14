use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the postmortem estimate calculation.
pub trait PostmortemEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePostmortemEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemEstimateModifier for LocalePostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PostmortemEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemEstimateModifier for FlaggedPostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemEstimateAdjuster {
    pub cap: f64,
}

impl PostmortemEstimateModifier for CapPostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemEstimateModifier for TieredPostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemEstimateAdjuster {
    pub minimum: f64,
}

impl PostmortemEstimateModifier for MinimumPostmortemEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the postmortem estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemEstimateEngine {
    pub steps: Vec<Rc<dyn PostmortemEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemEstimateEngine {
    pub fn price(&mut self, item: &PostmortemEstimate) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemEstimateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_estimate_engine(flags: &flags::Registry) -> PostmortemEstimateEngine {
    PostmortemEstimateEngine {
        steps: vec![
            Rc::new(LocalePostmortemEstimateAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.16),
                    ("north".to_string(), 1.25),
                    ("central".to_string(), 0.91),
                ]),
            }) as Rc<dyn PostmortemEstimateModifier>,
            Rc::new(FlaggedPostmortemEstimateAdjuster {
                flags: flags.clone(),
                flag: "postmortem-estimate-capped".to_string(),
                percent: -8.3,
            }),
            Rc::new(CapPostmortemEstimateAdjuster { cap: 9310.0 }),
            Rc::new(TieredPostmortemEstimateAdjuster {
                bounds: vec![2547.0, 7590.0],
                rates: vec![0.88, 1.14],
            }),
            Rc::new(MinimumPostmortemEstimateAdjuster { minimum: 17.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 61505.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_postmortem_estimate_engine(&flags::new_registry());
        let item = new_postmortem_estimate("code-892", "title-775");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
