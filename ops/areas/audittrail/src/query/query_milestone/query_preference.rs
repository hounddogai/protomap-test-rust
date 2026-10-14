use super::*;

use acme_kit::text;

/// Where a query preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QueryPreferenceStatus {
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

impl std::fmt::Display for QueryPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPreference {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub budget: f64,
    pub issuer: String,
    pub locale: String,
    pub key_id: String,
    pub score: f64,
    pub status: QueryPreferenceStatus,
}

impl QueryPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryPreferenceStatus) -> bool {
        match self.status {
            QueryPreferenceStatus::Open => {
                match next {
                    QueryPreferenceStatus::InReview | QueryPreferenceStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryPreferenceStatus::InReview => {
                return next == QueryPreferenceStatus::Resolved || next == QueryPreferenceStatus::Archived;
            }
            QueryPreferenceStatus::Resolved => {
                return next == QueryPreferenceStatus::Reopened || next == QueryPreferenceStatus::Archived;
            }
            QueryPreferenceStatus::Reopened => {
                match next {
                    QueryPreferenceStatus::Archived => {
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

    pub fn move_to(&mut self, next: QueryPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.score - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryPreference {
        QueryPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            budget: self.budget,
            issuer: self.issuer.clone(),
            locale: self.locale.clone(),
            key_id: self.key_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a query preference in status open.
pub fn new_query_preference(code: &str, label: &str) -> QueryPreference {
    QueryPreference {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        budget: 0.0,
        issuer: "".to_string(),
        locale: "".to_string(),
        key_id: "".to_string(),
        score: 0.0,
        status: QueryPreferenceStatus::Open,
    }
}
