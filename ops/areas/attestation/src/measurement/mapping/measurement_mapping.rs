use super::*;

use acme_kit::text;

/// Where a measurement mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl MeasurementMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for MeasurementMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMapping {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub title: String,
    pub category: String,
    pub segment: String,
    pub locale: String,
    pub external_ref: String,
    pub fingerprint: String,
    pub scope: String,
    pub status: MeasurementMappingStatus,
}

impl MeasurementMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementMappingStatus) -> bool {
        match self.status {
            MeasurementMappingStatus::Requested => {
                return next == MeasurementMappingStatus::Expired;
            }
            MeasurementMappingStatus::Granted => {
                match next {
                    MeasurementMappingStatus::Revoked | MeasurementMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementMappingStatus::Revoked => {
                match next {
                    MeasurementMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> MeasurementMapping {
        MeasurementMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            title: self.title.clone(),
            category: self.category.clone(),
            segment: self.segment.clone(),
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            fingerprint: self.fingerprint.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a measurement mapping in status requested.
pub fn new_measurement_mapping(code: &str, label: &str) -> MeasurementMapping {
    MeasurementMapping {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        title: "".to_string(),
        category: "".to_string(),
        segment: "".to_string(),
        locale: "".to_string(),
        external_ref: "".to_string(),
        fingerprint: "".to_string(),
        scope: "".to_string(),
        status: MeasurementMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_measurement_mapping("code-852", "label-950");
        assert_eq!(item.status, MeasurementMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_measurement_mapping("code-852", "label-950");
        if item.move_to(MeasurementMappingStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementMappingStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_measurement_mapping("code-852", "label-950");
        item.status = MeasurementMappingStatus::Expired;
        assert!(
            item.move_to(MeasurementMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
