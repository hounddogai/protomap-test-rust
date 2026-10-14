use super::*;

use acme_kit::text;

/// Where a escalation baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EscalationBaselineStatus {
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

impl std::fmt::Display for EscalationBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBaseline {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub key_id: String,
    pub external_ref: String,
    pub timezone: String,
    pub sequence: i64,
    pub summary: String,
    pub status: EscalationBaselineStatus,
}

impl EscalationBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationBaselineStatus) -> bool {
        match self.status {
            EscalationBaselineStatus::Open => {
                match next {
                    EscalationBaselineStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationBaselineStatus::InReview => {
                match next {
                    EscalationBaselineStatus::Resolved | EscalationBaselineStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationBaselineStatus::Resolved => {
                return next == EscalationBaselineStatus::Reopened || next == EscalationBaselineStatus::Archived;
            }
            EscalationBaselineStatus::Reopened => {
                return next == EscalationBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationBaseline {
        EscalationBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            key_id: self.key_id.clone(),
            external_ref: self.external_ref.clone(),
            timezone: self.timezone.clone(),
            sequence: self.sequence,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a escalation baseline in status open.
pub fn new_escalation_baseline(code: &str, label: &str) -> EscalationBaseline {
    EscalationBaseline {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        key_id: "".to_string(),
        external_ref: "".to_string(),
        timezone: "".to_string(),
        sequence: 0,
        summary: "".to_string(),
        status: EscalationBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_escalation_baseline("code-746", "label-130");
        assert_eq!(item.status, EscalationBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_escalation_baseline("code-746", "label-130");
        if item.move_to(EscalationBaselineStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationBaselineStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_escalation_baseline("code-746", "label-130");
        item.status = EscalationBaselineStatus::Archived;
        assert!(
            item.move_to(EscalationBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
