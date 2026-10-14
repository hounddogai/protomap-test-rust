use super::*;

use acme_kit::text;

/// Where a escalation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EscalationOverrideStatus {
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

impl std::fmt::Display for EscalationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation override as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationOverride {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub starts_at: i64,
    pub description: String,
    pub currency: String,
    pub burst: i64,
    pub priority: i64,
    pub fingerprint: String,
    pub status: EscalationOverrideStatus,
}

impl EscalationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationOverrideStatus) -> bool {
        match self.status {
            EscalationOverrideStatus::Planned => {
                return next == EscalationOverrideStatus::Completed || next == EscalationOverrideStatus::Cancelled;
            }
            EscalationOverrideStatus::Scheduled => {
                match next {
                    EscalationOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationOverrideStatus::InProgress => {
                return next == EscalationOverrideStatus::Completed;
            }
            EscalationOverrideStatus::Completed => {
                match next {
                    EscalationOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EscalationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.burst as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationOverride {
        EscalationOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            starts_at: self.starts_at,
            description: self.description.clone(),
            currency: self.currency.clone(),
            burst: self.burst,
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a escalation override in status planned.
pub fn new_escalation_override(code: &str, label: &str) -> EscalationOverride {
    EscalationOverride {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        starts_at: 0,
        description: "".to_string(),
        currency: "".to_string(),
        burst: 0,
        priority: 0,
        fingerprint: "".to_string(),
        status: EscalationOverrideStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_escalation_override("code-354", "label-458");
        assert_eq!(item.status, EscalationOverrideStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_escalation_override("code-354", "label-458");
        if item.move_to(EscalationOverrideStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationOverrideStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_escalation_override("code-354", "label-458");
        item.status = EscalationOverrideStatus::Cancelled;
        assert!(
            item.move_to(EscalationOverrideStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
