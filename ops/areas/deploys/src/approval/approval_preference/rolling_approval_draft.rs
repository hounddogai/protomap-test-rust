use super::*;

use acme_kit::text;

/// Where a rolling approval draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingApprovalDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingApprovalDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingApprovalDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling approval draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalDraft {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub sequence: i64,
    pub version: i64,
    pub margin: f64,
    pub status: RollingApprovalDraftStatus,
}

impl RollingApprovalDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingApprovalDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingApprovalDraftStatus) -> bool {
        match self.status {
            RollingApprovalDraftStatus::Proposed => {
                match next {
                    RollingApprovalDraftStatus::Approved | RollingApprovalDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingApprovalDraftStatus::Approved => {
                match next {
                    RollingApprovalDraftStatus::Rejected | RollingApprovalDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingApprovalDraftStatus::Rejected => {
                return next == RollingApprovalDraftStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingApprovalDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling approval draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.sequence as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingApprovalDraft {
        RollingApprovalDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            sequence: self.sequence,
            version: self.version,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling approval draft in status proposed.
pub fn new_rolling_approval_draft(code: &str, title: &str) -> RollingApprovalDraft {
    RollingApprovalDraft {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        sequence: 0,
        version: 0,
        margin: 0.0,
        status: RollingApprovalDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_approval_draft("code-145", "title-563");
        assert_eq!(item.status, RollingApprovalDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_approval_draft("code-145", "title-563");
        if item.move_to(RollingApprovalDraftStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingApprovalDraftStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_approval_draft("code-145", "title-563");
        item.status = RollingApprovalDraftStatus::Withdrawn;
        assert!(
            item.move_to(RollingApprovalDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
