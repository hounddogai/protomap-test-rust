use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual measurement batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualMeasurementBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualMeasurementBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualMeasurementBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual measurement batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementBatch {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub attributes: BTreeMap<String, String>,
    pub expires_at: i64,
    pub margin: f64,
    pub region: String,
    pub cost: f64,
    pub rate: f64,
    pub version: i64,
    pub status: ManualMeasurementBatchStatus,
}

impl ManualMeasurementBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualMeasurementBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualMeasurementBatchStatus) -> bool {
        match self.status {
            ManualMeasurementBatchStatus::Proposed => {
                match next {
                    ManualMeasurementBatchStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualMeasurementBatchStatus::Approved => {
                return next == ManualMeasurementBatchStatus::Rejected
                    || next == ManualMeasurementBatchStatus::Withdrawn;
            }
            ManualMeasurementBatchStatus::Rejected => {
                return next == ManualMeasurementBatchStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualMeasurementBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual measurement batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual measurement batch", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.cost + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualMeasurementBatch {
        ManualMeasurementBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            attributes: self.attributes.clone(),
            expires_at: self.expires_at,
            margin: self.margin,
            region: self.region.clone(),
            cost: self.cost,
            rate: self.rate,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual measurement batch in status proposed.
pub fn new_manual_measurement_batch(code: &str, label: &str) -> ManualMeasurementBatch {
    ManualMeasurementBatch {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        attributes: BTreeMap::new(),
        expires_at: 0,
        margin: 0.0,
        region: "".to_string(),
        cost: 0.0,
        rate: 0.0,
        version: 0,
        status: ManualMeasurementBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_measurement_batch("code-876", "label-417");
        assert_eq!(item.status, ManualMeasurementBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_manual_measurement_batch("code-876", "label-417");
        if item.move_to(ManualMeasurementBatchStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualMeasurementBatchStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_measurement_batch("code-876", "label-417");
        item.status = ManualMeasurementBatchStatus::Withdrawn;
        assert!(
            item.move_to(ManualMeasurementBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
