use super::*;

use acme_kit::text;

/// Where a primary timeline rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTimelineRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryTimelineRuleStatus {
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

impl std::fmt::Display for PrimaryTimelineRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary timeline rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTimelineRule {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub external_ref: String,
    pub namespace: String,
    pub grace_days: i64,
    pub label: String,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub status: PrimaryTimelineRuleStatus,
}

impl PrimaryTimelineRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTimelineRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTimelineRuleStatus) -> bool {
        match self.status {
            PrimaryTimelineRuleStatus::Planned => {
                match next {
                    PrimaryTimelineRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTimelineRuleStatus::Scheduled => {
                return next == PrimaryTimelineRuleStatus::InProgress || next == PrimaryTimelineRuleStatus::Completed;
            }
            PrimaryTimelineRuleStatus::InProgress => {
                return next == PrimaryTimelineRuleStatus::Completed || next == PrimaryTimelineRuleStatus::Cancelled;
            }
            PrimaryTimelineRuleStatus::Completed => {
                match next {
                    PrimaryTimelineRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryTimelineRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary timeline rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary timeline rule", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.grace_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTimelineRule {
        PrimaryTimelineRule {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            namespace: self.namespace.clone(),
            grace_days: self.grace_days,
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary timeline rule in status planned.
pub fn new_primary_timeline_rule(code: &str, title: &str) -> PrimaryTimelineRule {
    PrimaryTimelineRule {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        namespace: "".to_string(),
        grace_days: 0,
        label: "".to_string(),
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        status: PrimaryTimelineRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_timeline_rule("code-441", "title-947");
        assert_eq!(item.status, PrimaryTimelineRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_primary_timeline_rule("code-441", "title-947");
        if item.move_to(PrimaryTimelineRuleStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTimelineRuleStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_timeline_rule("code-441", "title-947");
        item.status = PrimaryTimelineRuleStatus::Cancelled;
        assert!(
            item.move_to(PrimaryTimelineRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
