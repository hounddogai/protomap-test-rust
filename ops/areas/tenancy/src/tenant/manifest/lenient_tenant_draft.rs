use super::*;

use acme_kit::text;

/// Where a lenient tenant draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTenantDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientTenantDraftStatus {
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

impl std::fmt::Display for LenientTenantDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient tenant draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantDraft {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub rotation_days: i64,
    pub ttl_seconds: i64,
    pub category: String,
    pub audience: String,
    pub status: LenientTenantDraftStatus,
}

impl LenientTenantDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTenantDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTenantDraftStatus) -> bool {
        match self.status {
            LenientTenantDraftStatus::Open => {
                return next == LenientTenantDraftStatus::InReview || next == LenientTenantDraftStatus::Resolved;
            }
            LenientTenantDraftStatus::InReview => {
                match next {
                    LenientTenantDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTenantDraftStatus::Resolved => {
                match next {
                    LenientTenantDraftStatus::Reopened | LenientTenantDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTenantDraftStatus::Reopened => {
                match next {
                    LenientTenantDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientTenantDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient tenant draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient tenant draft", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTenantDraft {
        LenientTenantDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            rotation_days: self.rotation_days,
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a lenient tenant draft in status open.
pub fn new_lenient_tenant_draft(code: &str, label: &str) -> LenientTenantDraft {
    LenientTenantDraft {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        rotation_days: 0,
        ttl_seconds: 0,
        category: "".to_string(),
        audience: "".to_string(),
        status: LenientTenantDraftStatus::Open,
    }
}
