use super::*;

use acme_kit::text;

/// Where a legacy measurement preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMeasurementPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyMeasurementPreferenceStatus {
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

impl std::fmt::Display for LegacyMeasurementPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy measurement preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementPreference {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub capacity: i64,
    pub contact_email: String,
    pub pinned: bool,
    pub ttl_seconds: i64,
    pub limit: i64,
    pub ceiling: f64,
    pub status: LegacyMeasurementPreferenceStatus,
}

impl LegacyMeasurementPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMeasurementPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMeasurementPreferenceStatus) -> bool {
        match self.status {
            LegacyMeasurementPreferenceStatus::Draft => {
                match next {
                    LegacyMeasurementPreferenceStatus::Suspended | LegacyMeasurementPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementPreferenceStatus::Pending => {
                match next {
                    LegacyMeasurementPreferenceStatus::Active | LegacyMeasurementPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementPreferenceStatus::Active => {
                match next {
                    LegacyMeasurementPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementPreferenceStatus::Suspended => {
                return next == LegacyMeasurementPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyMeasurementPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy measurement preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy measurement preference", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.maximum - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyMeasurementPreference {
        LegacyMeasurementPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            capacity: self.capacity,
            contact_email: self.contact_email.clone(),
            pinned: self.pinned,
            ttl_seconds: self.ttl_seconds,
            limit: self.limit,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy measurement preference in status draft.
pub fn new_legacy_measurement_preference(code: &str, name: &str) -> LegacyMeasurementPreference {
    LegacyMeasurementPreference {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        capacity: 0,
        contact_email: "".to_string(),
        pinned: false,
        ttl_seconds: 0,
        limit: 0,
        ceiling: 0.0,
        status: LegacyMeasurementPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_measurement_preference("code-786", "name-891");
        assert_eq!(item.status, LegacyMeasurementPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_legacy_measurement_preference("code-786", "name-891");
        if item.move_to(LegacyMeasurementPreferenceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyMeasurementPreferenceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_measurement_preference("code-786", "name-891");
        item.status = LegacyMeasurementPreferenceStatus::Closed;
        assert!(
            item.move_to(LegacyMeasurementPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
