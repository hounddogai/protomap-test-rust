use super::*;

use acme_kit::text;

/// Where a legacy rollout segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRolloutSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyRolloutSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyRolloutSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollout segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutSegment {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub key_id: String,
    pub quota_units: i64,
    pub country: String,
    pub fingerprint: String,
    pub port: i64,
    pub status: LegacyRolloutSegmentStatus,
}

impl LegacyRolloutSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRolloutSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRolloutSegmentStatus) -> bool {
        match self.status {
            LegacyRolloutSegmentStatus::Created => {
                return next == LegacyRolloutSegmentStatus::Retired;
            }
            LegacyRolloutSegmentStatus::Validated => {
                return next == LegacyRolloutSegmentStatus::Published;
            }
            LegacyRolloutSegmentStatus::Published => {
                match next {
                    LegacyRolloutSegmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyRolloutSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollout segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.quota_units as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRolloutSegment {
        LegacyRolloutSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            key_id: self.key_id.clone(),
            quota_units: self.quota_units,
            country: self.country.clone(),
            fingerprint: self.fingerprint.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy rollout segment in status created.
pub fn new_legacy_rollout_segment(code: &str, title: &str) -> LegacyRolloutSegment {
    LegacyRolloutSegment {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        key_id: "".to_string(),
        quota_units: 0,
        country: "".to_string(),
        fingerprint: "".to_string(),
        port: 0,
        status: LegacyRolloutSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_rollout_segment("code-84", "title-29");
        assert_eq!(item.status, LegacyRolloutSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_legacy_rollout_segment("code-84", "title-29");
        if item.move_to(LegacyRolloutSegmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRolloutSegmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_rollout_segment("code-84", "title-29");
        item.status = LegacyRolloutSegmentStatus::Retired;
        assert!(
            item.move_to(LegacyRolloutSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
