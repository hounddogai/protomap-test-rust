use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default forecast notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultForecastNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultForecastNoticeStatus {
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

impl std::fmt::Display for DefaultForecastNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default forecast notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastNotice {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub budget: f64,
    pub attributes: BTreeMap<String, String>,
    pub name: String,
    pub principal: String,
    pub archived: bool,
    pub cipher: String,
    pub status: DefaultForecastNoticeStatus,
}

impl DefaultForecastNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultForecastNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultForecastNoticeStatus) -> bool {
        match self.status {
            DefaultForecastNoticeStatus::Open => {
                return next == DefaultForecastNoticeStatus::InReview || next == DefaultForecastNoticeStatus::Resolved;
            }
            DefaultForecastNoticeStatus::InReview => {
                return next == DefaultForecastNoticeStatus::Resolved || next == DefaultForecastNoticeStatus::Archived;
            }
            DefaultForecastNoticeStatus::Resolved => {
                match next {
                    DefaultForecastNoticeStatus::Reopened | DefaultForecastNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultForecastNoticeStatus::Reopened => {
                match next {
                    DefaultForecastNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultForecastNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default forecast notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default forecast notice", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> DefaultForecastNotice {
        DefaultForecastNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            budget: self.budget,
            attributes: self.attributes.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            archived: self.archived,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default forecast notice in status open.
pub fn new_default_forecast_notice(code: &str, label: &str) -> DefaultForecastNotice {
    DefaultForecastNotice {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        budget: 0.0,
        attributes: BTreeMap::new(),
        name: "".to_string(),
        principal: "".to_string(),
        archived: false,
        cipher: "".to_string(),
        status: DefaultForecastNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_forecast_notice("code-788", "label-29");
        assert_eq!(item.status, DefaultForecastNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_default_forecast_notice("code-788", "label-29");
        if item.move_to(DefaultForecastNoticeStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultForecastNoticeStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_forecast_notice("code-788", "label-29");
        item.status = DefaultForecastNoticeStatus::Archived;
        assert!(
            item.move_to(DefaultForecastNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
