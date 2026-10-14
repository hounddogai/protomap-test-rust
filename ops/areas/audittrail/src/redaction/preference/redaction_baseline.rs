use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a redaction baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionBaselineStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RedactionBaselineStatus {
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

impl std::fmt::Display for RedactionBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBaseline {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub discount: f64,
    pub margin: f64,
    pub budget: f64,
    pub approved: bool,
    pub namespace: String,
    pub principal: String,
    pub region: String,
    pub status: RedactionBaselineStatus,
}

impl RedactionBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionBaselineStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionBaselineStatus) -> bool {
        match self.status {
            RedactionBaselineStatus::Draft => {
                match next {
                    RedactionBaselineStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionBaselineStatus::Pending => {
                match next {
                    RedactionBaselineStatus::Suspended | RedactionBaselineStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionBaselineStatus::Active => {
                return next == RedactionBaselineStatus::Suspended || next == RedactionBaselineStatus::Closed;
            }
            RedactionBaselineStatus::Suspended => {
                return next == RedactionBaselineStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn net_total(&self) -> f64 {
        ((self.margin - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionBaseline {
        RedactionBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            discount: self.discount,
            margin: self.margin,
            budget: self.budget,
            approved: self.approved,
            namespace: self.namespace.clone(),
            principal: self.principal.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a redaction baseline in status draft.
pub fn new_redaction_baseline(code: &str, label: &str) -> RedactionBaseline {
    RedactionBaseline {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        discount: 0.0,
        margin: 0.0,
        budget: 0.0,
        approved: false,
        namespace: "".to_string(),
        principal: "".to_string(),
        region: "".to_string(),
        status: RedactionBaselineStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_redaction_baseline("code-869", "label-668");
        assert_eq!(item.status, RedactionBaselineStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_redaction_baseline("code-869", "label-668");
        if item.move_to(RedactionBaselineStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionBaselineStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_redaction_baseline("code-869", "label-668");
        item.status = RedactionBaselineStatus::Closed;
        assert!(
            item.move_to(RedactionBaselineStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
