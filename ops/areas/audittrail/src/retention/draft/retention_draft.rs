use super::*;

use acme_kit::text;

/// Where a retention draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionDraftStatus {
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

impl std::fmt::Display for RetentionDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionDraft {
    pub code: String,
    pub name: String,
    pub region: String,
    pub retries: i64,
    pub ratio: f64,
    pub rate: f64,
    pub tenant_id: String,
    pub status: RetentionDraftStatus,
}

impl RetentionDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionDraftStatus) -> bool {
        match self.status {
            RetentionDraftStatus::New => {
                match next {
                    RetentionDraftStatus::Queued | RetentionDraftStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionDraftStatus::Queued => {
                return next == RetentionDraftStatus::Running || next == RetentionDraftStatus::Failed;
            }
            RetentionDraftStatus::Running => {
                return next == RetentionDraftStatus::Done;
            }
            RetentionDraftStatus::Done => {
                return next == RetentionDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ratio - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionDraft {
        RetentionDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            retries: self.retries,
            ratio: self.ratio,
            rate: self.rate,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a retention draft in status new.
pub fn new_retention_draft(code: &str, name: &str) -> RetentionDraft {
    RetentionDraft {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        retries: 0,
        ratio: 0.0,
        rate: 0.0,
        tenant_id: "".to_string(),
        status: RetentionDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_draft("code-535", "name-845");
        assert_eq!(item.status, RetentionDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_retention_draft("code-535", "name-845");
        if item.move_to(RetentionDraftStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionDraftStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_draft("code-535", "name-845");
        item.status = RetentionDraftStatus::Failed;
        assert!(
            item.move_to(RetentionDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
