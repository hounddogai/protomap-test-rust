use super::*;

use acme_kit::text;

/// Where a postmortem quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PostmortemQuotaStatus {
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

impl std::fmt::Display for PostmortemQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem quota as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemQuota {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub audience: String,
    pub scope: String,
    pub status: PostmortemQuotaStatus,
}

impl PostmortemQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemQuotaStatus) -> bool {
        match self.status {
            PostmortemQuotaStatus::Open => {
                match next {
                    PostmortemQuotaStatus::Resolved | PostmortemQuotaStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemQuotaStatus::InReview => {
                return next == PostmortemQuotaStatus::Reopened;
            }
            PostmortemQuotaStatus::Resolved => {
                match next {
                    PostmortemQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemQuotaStatus::Reopened => {
                match next {
                    PostmortemQuotaStatus::Archived => {
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

    pub fn move_to(&mut self, next: PostmortemQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> PostmortemQuota {
        PostmortemQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a postmortem quota in status open.
pub fn new_postmortem_quota(code: &str, label: &str) -> PostmortemQuota {
    PostmortemQuota {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        audience: "".to_string(),
        scope: "".to_string(),
        status: PostmortemQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_postmortem_quota("code-65", "label-918");
        assert_eq!(item.status, PostmortemQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_postmortem_quota("code-65", "label-918");
        if item.move_to(PostmortemQuotaStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemQuotaStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_postmortem_quota("code-65", "label-918");
        item.status = PostmortemQuotaStatus::Archived;
        assert!(
            item.move_to(PostmortemQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
