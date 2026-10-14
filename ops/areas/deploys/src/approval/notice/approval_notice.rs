use super::*;

use acme_kit::text;

/// Where a approval notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ApprovalNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ApprovalNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval notice as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNotice {
    pub code: String,
    pub label: String,
    pub title: String,
    pub retries: i64,
    pub reviewer: String,
    pub external_ref: String,
    pub threshold: f64,
    pub batch_size: i64,
    pub starts_at: i64,
    pub status: ApprovalNoticeStatus,
}

impl ApprovalNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalNoticeStatus) -> bool {
        match self.status {
            ApprovalNoticeStatus::Proposed => {
                match next {
                    ApprovalNoticeStatus::Approved | ApprovalNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalNoticeStatus::Approved => {
                match next {
                    ApprovalNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalNoticeStatus::Rejected => {
                return next == ApprovalNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval notice {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.threshold * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalNotice {
        ApprovalNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            retries: self.retries,
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            threshold: self.threshold,
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a approval notice in status proposed.
pub fn new_approval_notice(code: &str, label: &str) -> ApprovalNotice {
    ApprovalNotice {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        retries: 0,
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        threshold: 0.0,
        batch_size: 0,
        starts_at: 0,
        status: ApprovalNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_approval_notice("code-515", "label-653");
        assert_eq!(item.status, ApprovalNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_approval_notice("code-515", "label-653");
        if item.move_to(ApprovalNoticeStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalNoticeStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_approval_notice("code-515", "label-653");
        item.status = ApprovalNoticeStatus::Withdrawn;
        assert!(
            item.move_to(ApprovalNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
