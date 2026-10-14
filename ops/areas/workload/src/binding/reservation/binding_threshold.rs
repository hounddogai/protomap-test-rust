use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a binding threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BindingThresholdStatus {
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

impl std::fmt::Display for BindingThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingThreshold {
    pub code: String,
    pub title: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub duration_minutes: i64,
    pub source: String,
    pub summary: String,
    pub expires_at: i64,
    pub status: BindingThresholdStatus,
}

impl BindingThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingThresholdStatus) -> bool {
        match self.status {
            BindingThresholdStatus::Open => {
                return next == BindingThresholdStatus::Resolved;
            }
            BindingThresholdStatus::InReview => {
                match next {
                    BindingThresholdStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingThresholdStatus::Resolved => {
                match next {
                    BindingThresholdStatus::Reopened | BindingThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingThresholdStatus::Reopened => {
                match next {
                    BindingThresholdStatus::Archived => {
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

    pub fn move_to(&mut self, next: BindingThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding threshold {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
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

    pub fn projected_value(&self) -> f64 {
        ((self.expires_at as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingThreshold {
        BindingThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            duration_minutes: self.duration_minutes,
            source: self.source.clone(),
            summary: self.summary.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a binding threshold in status open.
pub fn new_binding_threshold(code: &str, title: &str) -> BindingThreshold {
    BindingThreshold {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        attributes: BTreeMap::new(),
        duration_minutes: 0,
        source: "".to_string(),
        summary: "".to_string(),
        expires_at: 0,
        status: BindingThresholdStatus::Open,
    }
}
