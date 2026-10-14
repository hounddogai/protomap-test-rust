use super::*;

use acme_kit::text;

/// Where a adaptive audit event entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAuditEventEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveAuditEventEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveAuditEventEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive audit event entry as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAuditEventEntry {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub port: i64,
    pub ratio: f64,
    pub status: AdaptiveAuditEventEntryStatus,
}

impl AdaptiveAuditEventEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAuditEventEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAuditEventEntryStatus) -> bool {
        match self.status {
            AdaptiveAuditEventEntryStatus::Proposed => {
                match next {
                    AdaptiveAuditEventEntryStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAuditEventEntryStatus::Approved => {
                return next == AdaptiveAuditEventEntryStatus::Withdrawn;
            }
            AdaptiveAuditEventEntryStatus::Rejected => {
                match next {
                    AdaptiveAuditEventEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveAuditEventEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive audit event entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ratio + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAuditEventEntry {
        AdaptiveAuditEventEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            port: self.port,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive audit event entry in status proposed.
pub fn new_adaptive_audit_event_entry(code: &str, label: &str) -> AdaptiveAuditEventEntry {
    AdaptiveAuditEventEntry {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        port: 0,
        ratio: 0.0,
        status: AdaptiveAuditEventEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_audit_event_entry("code-756", "label-538");
        assert_eq!(item.status, AdaptiveAuditEventEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_adaptive_audit_event_entry("code-756", "label-538");
        if item.move_to(AdaptiveAuditEventEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAuditEventEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_audit_event_entry("code-756", "label-538");
        item.status = AdaptiveAuditEventEntryStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveAuditEventEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
