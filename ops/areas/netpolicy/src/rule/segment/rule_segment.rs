use super::*;

use acme_kit::text;

/// Where a rule segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RuleSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RuleSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSegment {
    pub code: String,
    pub title: String,
    pub label: String,
    pub priority: i64,
    pub fingerprint: String,
    pub segment: String,
    pub timezone: String,
    pub grace_days: i64,
    pub active: bool,
    pub contact_email: String,
    pub locale: String,
    pub status: RuleSegmentStatus,
}

impl RuleSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleSegmentStatus) -> bool {
        match self.status {
            RuleSegmentStatus::Proposed => {
                return next == RuleSegmentStatus::Approved || next == RuleSegmentStatus::Withdrawn;
            }
            RuleSegmentStatus::Approved => {
                match next {
                    RuleSegmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleSegmentStatus::Rejected => {
                return next == RuleSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rule segment", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleSegment {
        RuleSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            segment: self.segment.clone(),
            timezone: self.timezone.clone(),
            grace_days: self.grace_days,
            active: self.active,
            contact_email: self.contact_email.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rule segment in status proposed.
pub fn new_rule_segment(code: &str, title: &str) -> RuleSegment {
    RuleSegment {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        priority: 0,
        fingerprint: "".to_string(),
        segment: "".to_string(),
        timezone: "".to_string(),
        grace_days: 0,
        active: false,
        contact_email: "".to_string(),
        locale: "".to_string(),
        status: RuleSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rule_segment("code-605", "title-88");
        assert_eq!(item.status, RuleSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rule_segment("code-605", "title-88");
        if item.move_to(RuleSegmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleSegmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rule_segment("code-605", "title-88");
        item.status = RuleSegmentStatus::Withdrawn;
        assert!(
            item.move_to(RuleSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
