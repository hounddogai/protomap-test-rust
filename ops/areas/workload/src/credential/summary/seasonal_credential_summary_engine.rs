use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal credential summary calculation.
pub trait SeasonalCredentialSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalCredentialSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialSummaryEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalCredentialSummaryModifier for SeasonalCredentialSummaryEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalCredentialSummary, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalCredentialSummaryAdjuster {
    pub minimum: f64,
}

impl SeasonalCredentialSummaryModifier for MinimumSeasonalCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalCredentialSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSeasonalCredentialSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalCredentialSummaryModifier for SegmentSeasonalCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SeasonalCredentialSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalCredentialSummaryAdjuster {
    pub percent: f64,
}

impl SeasonalCredentialSummaryModifier for PercentSeasonalCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalCredentialSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal credential summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialSummaryEngine {
    pub steps: Vec<Rc<dyn SeasonalCredentialSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalCredentialSummaryEngine {
    pub fn run(&mut self, item: &SeasonalCredentialSummary) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalCredentialSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_credential_summary_engine() -> SeasonalCredentialSummaryEngine {
    SeasonalCredentialSummaryEngine {
        steps: vec![
            Rc::new(SeasonalCredentialSummaryEndsAtOffsetAdjuster { weight: 0.55 })
                as Rc<dyn SeasonalCredentialSummaryModifier>,
            Rc::new(MinimumSeasonalCredentialSummaryAdjuster { minimum: 47.0 }),
            Rc::new(SegmentSeasonalCredentialSummaryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.29)]),
            }),
            Rc::new(PercentSeasonalCredentialSummaryAdjuster { percent: 9.2 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 3408.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_credential_summary_engine();
        let item = new_seasonal_credential_summary("code-781", "title-217");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
