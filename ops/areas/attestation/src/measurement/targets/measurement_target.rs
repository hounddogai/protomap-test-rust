use super::*;

use acme_kit::text;

/// Where a measurement target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MeasurementTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for MeasurementTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement target as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTarget {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub active: bool,
    pub currency: String,
    pub version: i64,
    pub key_id: String,
    pub channel: String,
    pub quota_units: i64,
    pub label: String,
    pub status: MeasurementTargetStatus,
}

impl MeasurementTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementTargetStatus) -> bool {
        match self.status {
            MeasurementTargetStatus::Draft => {
                return next == MeasurementTargetStatus::Suspended || next == MeasurementTargetStatus::Closed;
            }
            MeasurementTargetStatus::Pending => {
                return next == MeasurementTargetStatus::Active || next == MeasurementTargetStatus::Closed;
            }
            MeasurementTargetStatus::Active => {
                return next == MeasurementTargetStatus::Suspended;
            }
            MeasurementTargetStatus::Suspended => {
                match next {
                    MeasurementTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: MeasurementTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementTarget {
        MeasurementTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            active: self.active,
            currency: self.currency.clone(),
            version: self.version,
            key_id: self.key_id.clone(),
            channel: self.channel.clone(),
            quota_units: self.quota_units,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a measurement target in status draft.
pub fn new_measurement_target(code: &str, name: &str) -> MeasurementTarget {
    MeasurementTarget {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        active: false,
        currency: "".to_string(),
        version: 0,
        key_id: "".to_string(),
        channel: "".to_string(),
        quota_units: 0,
        label: "".to_string(),
        status: MeasurementTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_measurement_target("code-175", "name-970");
        assert_eq!(item.status, MeasurementTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_measurement_target("code-175", "name-970");
        if item.move_to(MeasurementTargetStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementTargetStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_measurement_target("code-175", "name-970");
        item.status = MeasurementTargetStatus::Closed;
        assert!(
            item.move_to(MeasurementTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
