use super::*;

use acme_kit::text;

/// Where a grant template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantTemplateStatus {
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

impl std::fmt::Display for GrantTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantTemplate {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub quota_units: i64,
    pub timezone: String,
    pub weight: f64,
    pub expires_at: i64,
    pub budget: f64,
    pub threshold: f64,
    pub sequence: i64,
    pub active: bool,
    pub status: GrantTemplateStatus,
}

impl GrantTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantTemplateStatus) -> bool {
        match self.status {
            GrantTemplateStatus::Open => {
                return next == GrantTemplateStatus::Resolved || next == GrantTemplateStatus::Archived;
            }
            GrantTemplateStatus::InReview => {
                return next == GrantTemplateStatus::Reopened || next == GrantTemplateStatus::Archived;
            }
            GrantTemplateStatus::Resolved => {
                return next == GrantTemplateStatus::Reopened;
            }
            GrantTemplateStatus::Reopened => {
                return next == GrantTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant template {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.budget - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantTemplate {
        GrantTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            quota_units: self.quota_units,
            timezone: self.timezone.clone(),
            weight: self.weight,
            expires_at: self.expires_at,
            budget: self.budget,
            threshold: self.threshold,
            sequence: self.sequence,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a grant template in status open.
pub fn new_grant_template(code: &str, name: &str) -> GrantTemplate {
    GrantTemplate {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        quota_units: 0,
        timezone: "".to_string(),
        weight: 0.0,
        expires_at: 0,
        budget: 0.0,
        threshold: 0.0,
        sequence: 0,
        active: false,
        status: GrantTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_template("code-281", "name-376");
        assert_eq!(item.status, GrantTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_grant_template("code-281", "name-376");
        if item.move_to(GrantTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_template("code-281", "name-376");
        item.status = GrantTemplateStatus::Archived;
        assert!(
            item.move_to(GrantTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
