use super::*;

use acme_kit::text;

/// Where a verifier agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierAgreementStatus {
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

impl std::fmt::Display for VerifierAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAgreement {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub phone: String,
    pub version: i64,
    pub ends_at: i64,
    pub maximum: f64,
    pub segment: String,
    pub status: VerifierAgreementStatus,
}

impl VerifierAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierAgreementStatus) -> bool {
        match self.status {
            VerifierAgreementStatus::Planned => {
                return next == VerifierAgreementStatus::Scheduled || next == VerifierAgreementStatus::Cancelled;
            }
            VerifierAgreementStatus::Scheduled => {
                return next == VerifierAgreementStatus::Completed;
            }
            VerifierAgreementStatus::InProgress => {
                match next {
                    VerifierAgreementStatus::Completed | VerifierAgreementStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierAgreementStatus::Completed => {
                match next {
                    VerifierAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: VerifierAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier agreement", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ends_at as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierAgreement {
        VerifierAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            phone: self.phone.clone(),
            version: self.version,
            ends_at: self.ends_at,
            maximum: self.maximum,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a verifier agreement in status planned.
pub fn new_verifier_agreement(code: &str, title: &str) -> VerifierAgreement {
    VerifierAgreement {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        phone: "".to_string(),
        version: 0,
        ends_at: 0,
        maximum: 0.0,
        segment: "".to_string(),
        status: VerifierAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_verifier_agreement("code-310", "title-157");
        assert_eq!(item.status, VerifierAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_verifier_agreement("code-310", "title-157");
        if item.move_to(VerifierAgreementStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierAgreementStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_verifier_agreement("code-310", "title-157");
        item.status = VerifierAgreementStatus::Cancelled;
        assert!(
            item.move_to(VerifierAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
