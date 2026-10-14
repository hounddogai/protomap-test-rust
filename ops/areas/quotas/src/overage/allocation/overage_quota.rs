use super::*;

use acme_kit::text;

/// Where a overage quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OverageQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for OverageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageQuota {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub unit: String,
    pub starts_at: i64,
    pub external_ref: String,
    pub notes: String,
    pub key_id: String,
    pub attempts: i64,
    pub phone: String,
    pub status: OverageQuotaStatus,
}

impl OverageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageQuotaStatus) -> bool {
        match self.status {
            OverageQuotaStatus::Open => {
                return next == OverageQuotaStatus::Resolved || next == OverageQuotaStatus::Archived;
            }
            OverageQuotaStatus::InReview => {
                match next {
                    OverageQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageQuotaStatus::Resolved => {
                return next == OverageQuotaStatus::Reopened;
            }
            OverageQuotaStatus::Reopened => {
                return next == OverageQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.updated_at as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageQuota {
        OverageQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            external_ref: self.external_ref.clone(),
            notes: self.notes.clone(),
            key_id: self.key_id.clone(),
            attempts: self.attempts,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a overage quota in status open.
pub fn new_overage_quota(code: &str, name: &str) -> OverageQuota {
    OverageQuota {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        unit: "".to_string(),
        starts_at: 0,
        external_ref: "".to_string(),
        notes: "".to_string(),
        key_id: "".to_string(),
        attempts: 0,
        phone: "".to_string(),
        status: OverageQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_overage_quota("code-180", "name-667");
        assert_eq!(item.status, OverageQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_overage_quota("code-180", "name-667");
        if item.move_to(OverageQuotaStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageQuotaStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_overage_quota("code-180", "name-667");
        item.status = OverageQuotaStatus::Archived;
        assert!(item.move_to(OverageQuotaStatus::Open).is_err(), "expected move to fail");
    }
}
