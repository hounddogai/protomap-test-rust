use super::*;

use acme_kit::text;

/// Where a legacy freeze audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFreezeAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyFreezeAuditStatus {
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

impl std::fmt::Display for LegacyFreezeAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy freeze audit as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeAudit {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub floor_value: f64,
    pub quantity: i64,
    pub pinned: bool,
    pub phone: String,
    pub issuer: String,
    pub notes: String,
    pub description: String,
    pub expires_at: i64,
    pub status: LegacyFreezeAuditStatus,
}

impl LegacyFreezeAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFreezeAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFreezeAuditStatus) -> bool {
        match self.status {
            LegacyFreezeAuditStatus::Open => {
                return next == LegacyFreezeAuditStatus::Resolved;
            }
            LegacyFreezeAuditStatus::InReview => {
                match next {
                    LegacyFreezeAuditStatus::Reopened | LegacyFreezeAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFreezeAuditStatus::Resolved => {
                match next {
                    LegacyFreezeAuditStatus::Reopened | LegacyFreezeAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFreezeAuditStatus::Reopened => {
                return next == LegacyFreezeAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFreezeAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy freeze audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy freeze audit", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.starts_at as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFreezeAudit {
        LegacyFreezeAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            floor_value: self.floor_value,
            quantity: self.quantity,
            pinned: self.pinned,
            phone: self.phone.clone(),
            issuer: self.issuer.clone(),
            notes: self.notes.clone(),
            description: self.description.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy freeze audit in status open.
pub fn new_legacy_freeze_audit(code: &str, title: &str) -> LegacyFreezeAudit {
    LegacyFreezeAudit {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        floor_value: 0.0,
        quantity: 0,
        pinned: false,
        phone: "".to_string(),
        issuer: "".to_string(),
        notes: "".to_string(),
        description: "".to_string(),
        expires_at: 0,
        status: LegacyFreezeAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_freeze_audit("code-876", "title-87");
        assert_eq!(item.status, LegacyFreezeAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_legacy_freeze_audit("code-876", "title-87");
        if item.move_to(LegacyFreezeAuditStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFreezeAuditStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_freeze_audit("code-876", "title-87");
        item.status = LegacyFreezeAuditStatus::Archived;
        assert!(
            item.move_to(LegacyFreezeAuditStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
