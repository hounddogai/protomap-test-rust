use super::*;

use acme_kit::text;

/// Where a regional escalation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalEscalationSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalEscalationSegmentStatus {
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

impl std::fmt::Display for RegionalEscalationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional escalation segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationSegment {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub discount: f64,
    pub audience: String,
    pub host: String,
    pub minimum: f64,
    pub description: String,
    pub duration_minutes: i64,
    pub status: RegionalEscalationSegmentStatus,
}

impl RegionalEscalationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalEscalationSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalEscalationSegmentStatus) -> bool {
        match self.status {
            RegionalEscalationSegmentStatus::Open => {
                return next == RegionalEscalationSegmentStatus::Resolved
                    || next == RegionalEscalationSegmentStatus::Archived;
            }
            RegionalEscalationSegmentStatus::InReview => {
                match next {
                    RegionalEscalationSegmentStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEscalationSegmentStatus::Resolved => {
                match next {
                    RegionalEscalationSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEscalationSegmentStatus::Reopened => {
                return next == RegionalEscalationSegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalEscalationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional escalation segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalEscalationSegment {
        RegionalEscalationSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            discount: self.discount,
            audience: self.audience.clone(),
            host: self.host.clone(),
            minimum: self.minimum,
            description: self.description.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional escalation segment in status open.
pub fn new_regional_escalation_segment(code: &str, name: &str) -> RegionalEscalationSegment {
    RegionalEscalationSegment {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        discount: 0.0,
        audience: "".to_string(),
        host: "".to_string(),
        minimum: 0.0,
        description: "".to_string(),
        duration_minutes: 0,
        status: RegionalEscalationSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_escalation_segment("code-653", "name-722");
        assert_eq!(item.status, RegionalEscalationSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_regional_escalation_segment("code-653", "name-722");
        if item.move_to(RegionalEscalationSegmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalEscalationSegmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_escalation_segment("code-653", "name-722");
        item.status = RegionalEscalationSegmentStatus::Archived;
        assert!(
            item.move_to(RegionalEscalationSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
