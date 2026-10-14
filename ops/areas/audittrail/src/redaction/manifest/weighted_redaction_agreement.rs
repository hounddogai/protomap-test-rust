use super::*;

use acme_kit::text;

/// Where a weighted redaction agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRedactionAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedRedactionAgreementStatus {
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

impl std::fmt::Display for WeightedRedactionAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted redaction agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionAgreement {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub active: bool,
    pub updated_at: i64,
    pub retries: i64,
    pub reviewer: String,
    pub timezone: String,
    pub status: WeightedRedactionAgreementStatus,
}

impl WeightedRedactionAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRedactionAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRedactionAgreementStatus) -> bool {
        match self.status {
            WeightedRedactionAgreementStatus::Planned => {
                return next == WeightedRedactionAgreementStatus::InProgress;
            }
            WeightedRedactionAgreementStatus::Scheduled => {
                match next {
                    WeightedRedactionAgreementStatus::Completed | WeightedRedactionAgreementStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRedactionAgreementStatus::InProgress => {
                return next == WeightedRedactionAgreementStatus::Completed
                    || next == WeightedRedactionAgreementStatus::Cancelled;
            }
            WeightedRedactionAgreementStatus::Completed => {
                return next == WeightedRedactionAgreementStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRedactionAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted redaction agreement {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.updated_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRedactionAgreement {
        WeightedRedactionAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            active: self.active,
            updated_at: self.updated_at,
            retries: self.retries,
            reviewer: self.reviewer.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted redaction agreement in status planned.
pub fn new_weighted_redaction_agreement(code: &str, label: &str) -> WeightedRedactionAgreement {
    WeightedRedactionAgreement {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        active: false,
        updated_at: 0,
        retries: 0,
        reviewer: "".to_string(),
        timezone: "".to_string(),
        status: WeightedRedactionAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_redaction_agreement("code-200", "label-476");
        assert_eq!(item.status, WeightedRedactionAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_weighted_redaction_agreement("code-200", "label-476");
        if item.move_to(WeightedRedactionAgreementStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRedactionAgreementStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_redaction_agreement("code-200", "label-476");
        item.status = WeightedRedactionAgreementStatus::Cancelled;
        assert!(
            item.move_to(WeightedRedactionAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
