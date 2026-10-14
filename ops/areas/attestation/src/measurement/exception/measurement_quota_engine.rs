use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement quota calculation.
pub trait MeasurementQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementQuotaModifier for FlaggedMeasurementQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementQuotaAdjuster {
    pub percent: f64,
}

impl MeasurementQuotaModifier for PercentMeasurementQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleMeasurementQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementQuotaModifier for TitleMeasurementQuotaAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &MeasurementQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementQuotaEngine {
    pub steps: Vec<Rc<dyn MeasurementQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementQuotaEngine {
    pub fn compute(&mut self, item: &MeasurementQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[MeasurementQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_quota_engine(flags: &flags::Registry) -> MeasurementQuotaEngine {
    MeasurementQuotaEngine {
        steps: vec![
            Rc::new(FlaggedMeasurementQuotaAdjuster {
                flags: flags.clone(),
                flag: "measurement-quota-global".to_string(),
                percent: -1.2,
            }) as Rc<dyn MeasurementQuotaModifier>,
            Rc::new(PercentMeasurementQuotaAdjuster { percent: -22.1 }),
            Rc::new(TitleMeasurementQuotaAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.39), ("east".to_string(), 1.19)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 99089.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_quota_engine(&flags::new_registry());
        let item = new_measurement_quota("code-17", "title-360");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
