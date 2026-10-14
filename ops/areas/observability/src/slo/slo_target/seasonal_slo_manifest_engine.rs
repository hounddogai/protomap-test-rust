use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the seasonal slo manifest calculation.
pub trait SeasonalSloManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalSloManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleSeasonalSloManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalSloManifestFactor for LocaleSeasonalSloManifestAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &SeasonalSloManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloManifestWeightOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalSloManifestFactor for SeasonalSloManifestWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalSloManifest, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalSloManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalSloManifestFactor for FlaggedSeasonalSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalSloManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalSloManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalSloManifestFactor for TieredSeasonalSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalSloManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal slo manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalSloManifestEngine {
    pub steps: Vec<Rc<dyn SeasonalSloManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalSloManifestEngine {
    pub fn run(&mut self, item: &SeasonalSloManifest) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalSloManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalSloManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_slo_manifest_engine(flags: &flags::Registry) -> SeasonalSloManifestEngine {
    SeasonalSloManifestEngine {
        steps: vec![
            Rc::new(LocaleSeasonalSloManifestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.88)]),
            }) as Rc<dyn SeasonalSloManifestFactor>,
            Rc::new(SeasonalSloManifestWeightOffsetAdjuster { weight: 0.82 }),
            Rc::new(FlaggedSeasonalSloManifestAdjuster {
                flags: flags.clone(),
                flag: "seasonal-slo-manifest-seasonal".to_string(),
                percent: 0.7,
            }),
            Rc::new(TieredSeasonalSloManifestAdjuster {
                bounds: vec![3629.0, 4644.0, 4663.0, 8565.0],
                rates: vec![0.94, 1.09, 1.09, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 81858.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_slo_manifest_engine(&flags::new_registry());
        let item = new_seasonal_slo_manifest("code-906", "label-612");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
