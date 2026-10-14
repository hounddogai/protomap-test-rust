use super::*;

use acme_kit::text;

/// Where a verifier quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl VerifierQuotaStatus {
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

impl std::fmt::Display for VerifierQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierQuota {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub country: String,
    pub weight: f64,
    pub issuer: String,
    pub archived: bool,
    pub status: VerifierQuotaStatus,
}

impl VerifierQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierQuotaStatus) -> bool {
        match self.status {
            VerifierQuotaStatus::Open => {
                match next {
                    VerifierQuotaStatus::InReview | VerifierQuotaStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierQuotaStatus::InReview => {
                return next == VerifierQuotaStatus::Reopened || next == VerifierQuotaStatus::Archived;
            }
            VerifierQuotaStatus::Resolved => {
                return next == VerifierQuotaStatus::Reopened || next == VerifierQuotaStatus::Archived;
            }
            VerifierQuotaStatus::Reopened => {
                return next == VerifierQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.burst as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierQuota {
        VerifierQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            country: self.country.clone(),
            weight: self.weight,
            issuer: self.issuer.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a verifier quota in status open.
pub fn new_verifier_quota(code: &str, label: &str) -> VerifierQuota {
    VerifierQuota {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        country: "".to_string(),
        weight: 0.0,
        issuer: "".to_string(),
        archived: false,
        status: VerifierQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_verifier_quota("code-802", "label-920");
        assert_eq!(item.status, VerifierQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_verifier_quota("code-802", "label-920");
        if item.move_to(VerifierQuotaStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierQuotaStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_verifier_quota("code-802", "label-920");
        item.status = VerifierQuotaStatus::Archived;
        assert!(
            item.move_to(VerifierQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
