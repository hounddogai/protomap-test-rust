use super::*;

use acme_kit::text;

/// Where a primary ttl policy template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTtlPolicyTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryTtlPolicyTemplateStatus {
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

impl std::fmt::Display for PrimaryTtlPolicyTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary ttl policy template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyTemplate {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub port: i64,
    pub limit: i64,
    pub margin: f64,
    pub status: PrimaryTtlPolicyTemplateStatus,
}

impl PrimaryTtlPolicyTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTtlPolicyTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTtlPolicyTemplateStatus) -> bool {
        match self.status {
            PrimaryTtlPolicyTemplateStatus::Open => {
                match next {
                    PrimaryTtlPolicyTemplateStatus::Resolved | PrimaryTtlPolicyTemplateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyTemplateStatus::InReview => {
                return next == PrimaryTtlPolicyTemplateStatus::Resolved
                    || next == PrimaryTtlPolicyTemplateStatus::Reopened;
            }
            PrimaryTtlPolicyTemplateStatus::Resolved => {
                match next {
                    PrimaryTtlPolicyTemplateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyTemplateStatus::Reopened => {
                return next == PrimaryTtlPolicyTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTtlPolicyTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary ttl policy template {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.created_at as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTtlPolicyTemplate {
        PrimaryTtlPolicyTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            port: self.port,
            limit: self.limit,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary ttl policy template in status open.
pub fn new_primary_ttl_policy_template(code: &str, title: &str) -> PrimaryTtlPolicyTemplate {
    PrimaryTtlPolicyTemplate {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        port: 0,
        limit: 0,
        margin: 0.0,
        status: PrimaryTtlPolicyTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_ttl_policy_template("code-697", "title-348");
        assert_eq!(item.status, PrimaryTtlPolicyTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_primary_ttl_policy_template("code-697", "title-348");
        if item.move_to(PrimaryTtlPolicyTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTtlPolicyTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_ttl_policy_template("code-697", "title-348");
        item.status = PrimaryTtlPolicyTemplateStatus::Archived;
        assert!(
            item.move_to(PrimaryTtlPolicyTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
