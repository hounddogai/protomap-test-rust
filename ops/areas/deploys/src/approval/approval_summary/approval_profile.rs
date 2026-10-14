use super::*;

use acme_kit::text;

/// Where a approval profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ApprovalProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ApprovalProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProfile {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub created_at: i64,
    pub locale: String,
    pub status: ApprovalProfileStatus,
}

impl ApprovalProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalProfileStatus) -> bool {
        match self.status {
            ApprovalProfileStatus::New => {
                return next == ApprovalProfileStatus::Queued || next == ApprovalProfileStatus::Done;
            }
            ApprovalProfileStatus::Queued => {
                match next {
                    ApprovalProfileStatus::Running | ApprovalProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalProfileStatus::Running => {
                return next == ApprovalProfileStatus::Done;
            }
            ApprovalProfileStatus::Done => {
                match next {
                    ApprovalProfileStatus::Failed => {
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

    pub fn move_to(&mut self, next: ApprovalProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval profile {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.created_at as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalProfile {
        ApprovalProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            created_at: self.created_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a approval profile in status new.
pub fn new_approval_profile(code: &str, label: &str) -> ApprovalProfile {
    ApprovalProfile {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        created_at: 0,
        locale: "".to_string(),
        status: ApprovalProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_approval_profile("code-419", "label-703");
        assert_eq!(item.status, ApprovalProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_approval_profile("code-419", "label-703");
        if item.move_to(ApprovalProfileStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalProfileStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_approval_profile("code-419", "label-703");
        item.status = ApprovalProfileStatus::Failed;
        assert!(
            item.move_to(ApprovalProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
