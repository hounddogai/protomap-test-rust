use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted federation estimate calculation.
pub trait WeightedFederationEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFederationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationEstimateCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFederationEstimateAdjuster for WeightedFederationEstimateCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFederationEstimate, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFederationEstimateAdjuster {
    pub step: f64,
}

impl WeightedFederationEstimateAdjuster for RoundingWeightedFederationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFederationEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedFederationEstimateAdjuster {
    pub percent: f64,
}

impl WeightedFederationEstimateAdjuster for PercentWeightedFederationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedFederationEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedFederationEstimateAdjuster {
    pub minimum: f64,
}

impl WeightedFederationEstimateAdjuster for MinimumWeightedFederationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedFederationEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFederationEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFederationEstimateAdjuster for FlaggedWeightedFederationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFederationEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedFederationEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedFederationEstimateAdjuster for TieredWeightedFederationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedFederationEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted federation estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFederationEstimateEngine {
    pub steps: Vec<Rc<dyn WeightedFederationEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFederationEstimateEngine {
    pub fn evaluate(&mut self, item: &WeightedFederationEstimate) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFederationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_federation_estimate_engine(flags: &flags::Registry) -> WeightedFederationEstimateEngine {
    WeightedFederationEstimateEngine {
        steps: vec![
            Rc::new(WeightedFederationEstimateCreatedAtOffsetAdjuster { weight: 1.08 })
                as Rc<dyn WeightedFederationEstimateAdjuster>,
            Rc::new(RoundingWeightedFederationEstimateAdjuster { step: 0.05 }),
            Rc::new(PercentWeightedFederationEstimateAdjuster { percent: 17.7 }),
            Rc::new(MinimumWeightedFederationEstimateAdjuster { minimum: 1.8 }),
            Rc::new(FlaggedWeightedFederationEstimateAdjuster {
                flags: flags.clone(),
                flag: "weighted-federation-estimate-primary".to_string(),
                percent: -3.9,
            }),
            Rc::new(TieredWeightedFederationEstimateAdjuster {
                bounds: vec![442.0, 5372.0, 5508.0, 6922.0],
                rates: vec![1.14, 1.17, 1.05, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 15705.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_federation_estimate_engine(&flags::new_registry());
        let item = new_weighted_federation_estimate("code-596", "label-296");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
