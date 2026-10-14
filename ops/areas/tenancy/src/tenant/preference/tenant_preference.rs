use super::*;

use acme_kit::text;

/// Where a tenant preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantPreferenceStatus {
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

impl std::fmt::Display for TenantPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantPreference {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub visible: bool,
    pub ratio: f64,
    pub country: String,
    pub status: TenantPreferenceStatus,
}

impl TenantPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantPreferenceStatus) -> bool {
        match self.status {
            TenantPreferenceStatus::Open => {
                return next == TenantPreferenceStatus::InReview;
            }
            TenantPreferenceStatus::InReview => {
                return next == TenantPreferenceStatus::Resolved;
            }
            TenantPreferenceStatus::Resolved => {
                return next == TenantPreferenceStatus::Reopened || next == TenantPreferenceStatus::Archived;
            }
            TenantPreferenceStatus::Reopened => {
                return next == TenantPreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tenant preference", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.port as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantPreference {
        TenantPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            visible: self.visible,
            ratio: self.ratio,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tenant preference in status open.
pub fn new_tenant_preference(code: &str, name: &str) -> TenantPreference {
    TenantPreference {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        visible: false,
        ratio: 0.0,
        country: "".to_string(),
        status: TenantPreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_preference("code-849", "name-753");
        assert_eq!(item.status, TenantPreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tenant_preference("code-849", "name-753");
        if item.move_to(TenantPreferenceStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantPreferenceStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_preference("code-849", "name-753");
        item.status = TenantPreferenceStatus::Archived;
        assert!(
            item.move_to(TenantPreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
