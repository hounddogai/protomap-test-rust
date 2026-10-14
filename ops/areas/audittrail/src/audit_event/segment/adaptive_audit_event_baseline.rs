use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a adaptive audit event baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAuditEventBaselineStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveAuditEventBaselineStatus {
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

impl std::fmt::Display for AdaptiveAuditEventBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive audit event baseline as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventBaseline {
    pub code: String,
    pub label: String,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub reviewer: String,
    pub maximum: f64,
    pub priority: i64,
    pub status: AdaptiveAuditEventBaselineStatus,
}

impl AdaptiveAuditEventBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAuditEventBaselineStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAuditEventBaselineStatus) -> bool {
        match self.status {
            AdaptiveAuditEventBaselineStatus::Draft => {
                return next == AdaptiveAuditEventBaselineStatus::Pending
                    || next == AdaptiveAuditEventBaselineStatus::Suspended;
            }
            AdaptiveAuditEventBaselineStatus::Pending => {
                return next == AdaptiveAuditEventBaselineStatus::Active
                    || next == AdaptiveAuditEventBaselineStatus::Suspended;
            }
            AdaptiveAuditEventBaselineStatus::Active => {
                return next == AdaptiveAuditEventBaselineStatus::Suspended;
            }
            AdaptiveAuditEventBaselineStatus::Suspended => {
                match next {
                    AdaptiveAuditEventBaselineStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveAuditEventBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive audit event baseline {} cannot move from {} to {}",
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

    pub fn net_exposure(&self) -> f64 {
        ((self.priority as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAuditEventBaseline {
        AdaptiveAuditEventBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            reviewer: self.reviewer.clone(),
            maximum: self.maximum,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive audit event baseline in status draft.
pub fn new_adaptive_audit_event_baseline(code: &str, label: &str) -> AdaptiveAuditEventBaseline {
    AdaptiveAuditEventBaseline {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        attributes: BTreeMap::new(),
        reviewer: "".to_string(),
        maximum: 0.0,
        priority: 0,
        status: AdaptiveAuditEventBaselineStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_audit_event_baseline("code-631", "label-860");
        assert_eq!(item.status, AdaptiveAuditEventBaselineStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_adaptive_audit_event_baseline("code-631", "label-860");
        if item.move_to(AdaptiveAuditEventBaselineStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAuditEventBaselineStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_audit_event_baseline("code-631", "label-860");
        item.status = AdaptiveAuditEventBaselineStatus::Closed;
        assert!(
            item.move_to(AdaptiveAuditEventBaselineStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
