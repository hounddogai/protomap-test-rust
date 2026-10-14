use super::*;

use acme_kit::text;

/// Where a rolling release record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingReleaseRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingReleaseRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingReleaseRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling release record as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseRecord {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub created_at: i64,
    pub external_ref: String,
    pub contact_email: String,
    pub category: String,
    pub description: String,
    pub status: RollingReleaseRecordStatus,
}

impl RollingReleaseRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingReleaseRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingReleaseRecordStatus) -> bool {
        match self.status {
            RollingReleaseRecordStatus::Proposed => {
                match next {
                    RollingReleaseRecordStatus::Approved | RollingReleaseRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingReleaseRecordStatus::Approved => {
                return next == RollingReleaseRecordStatus::Rejected || next == RollingReleaseRecordStatus::Withdrawn;
            }
            RollingReleaseRecordStatus::Rejected => {
                return next == RollingReleaseRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingReleaseRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling release record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling release record", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.created_at as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingReleaseRecord {
        RollingReleaseRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            created_at: self.created_at,
            external_ref: self.external_ref.clone(),
            contact_email: self.contact_email.clone(),
            category: self.category.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling release record in status proposed.
pub fn new_rolling_release_record(code: &str, label: &str) -> RollingReleaseRecord {
    RollingReleaseRecord {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        created_at: 0,
        external_ref: "".to_string(),
        contact_email: "".to_string(),
        category: "".to_string(),
        description: "".to_string(),
        status: RollingReleaseRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_release_record("code-188", "label-811");
        assert_eq!(item.status, RollingReleaseRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_release_record("code-188", "label-811");
        if item.move_to(RollingReleaseRecordStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingReleaseRecordStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_release_record("code-188", "label-811");
        item.status = RollingReleaseRecordStatus::Withdrawn;
        assert!(
            item.move_to(RollingReleaseRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
