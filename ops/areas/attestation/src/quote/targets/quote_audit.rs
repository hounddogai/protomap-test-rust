use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a quote audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QuoteAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for QuoteAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAudit {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub budget: f64,
    pub burst: i64,
    pub attributes: BTreeMap<String, String>,
    pub attempts: i64,
    pub status: QuoteAuditStatus,
}

impl QuoteAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteAuditStatus) -> bool {
        match self.status {
            QuoteAuditStatus::Planned => {
                return next == QuoteAuditStatus::InProgress || next == QuoteAuditStatus::Cancelled;
            }
            QuoteAuditStatus::Scheduled => {
                match next {
                    QuoteAuditStatus::InProgress | QuoteAuditStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteAuditStatus::InProgress => {
                match next {
                    QuoteAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteAuditStatus::Completed => {
                return next == QuoteAuditStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "quote audit", self.code)
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

    pub fn adjusted_total(&self) -> f64 {
        ((self.budget + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteAudit {
        QuoteAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            budget: self.budget,
            burst: self.burst,
            attributes: self.attributes.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a quote audit in status planned.
pub fn new_quote_audit(code: &str, label: &str) -> QuoteAudit {
    QuoteAudit {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        budget: 0.0,
        burst: 0,
        attributes: BTreeMap::new(),
        attempts: 0,
        status: QuoteAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_quote_audit("code-527", "label-363");
        assert_eq!(item.status, QuoteAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_quote_audit("code-527", "label-363");
        if item.move_to(QuoteAuditStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteAuditStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_quote_audit("code-527", "label-363");
        item.status = QuoteAuditStatus::Cancelled;
        assert!(
            item.move_to(QuoteAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
