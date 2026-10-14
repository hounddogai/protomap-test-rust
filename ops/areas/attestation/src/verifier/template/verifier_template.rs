use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a verifier template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierTemplateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl VerifierTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for VerifierTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTemplate {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub priority: i64,
    pub visible: bool,
    pub amount: f64,
    pub attributes: BTreeMap<String, String>,
    pub ends_at: i64,
    pub status: VerifierTemplateStatus,
}

impl VerifierTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierTemplateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierTemplateStatus) -> bool {
        match self.status {
            VerifierTemplateStatus::Draft => {
                return next == VerifierTemplateStatus::Active;
            }
            VerifierTemplateStatus::Pending => {
                return next == VerifierTemplateStatus::Active;
            }
            VerifierTemplateStatus::Active => {
                return next == VerifierTemplateStatus::Closed;
            }
            VerifierTemplateStatus::Suspended => {
                match next {
                    VerifierTemplateStatus::Closed => {
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

    pub fn move_to(&mut self, next: VerifierTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier template", self.code)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.amount + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierTemplate {
        VerifierTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            priority: self.priority,
            visible: self.visible,
            amount: self.amount,
            attributes: self.attributes.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a verifier template in status draft.
pub fn new_verifier_template(code: &str, title: &str) -> VerifierTemplate {
    VerifierTemplate {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        priority: 0,
        visible: false,
        amount: 0.0,
        attributes: BTreeMap::new(),
        ends_at: 0,
        status: VerifierTemplateStatus::Draft,
    }
}
