use super::*;

use acme_kit::text;

/// Where a egress agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EgressAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for EgressAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress agreement as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressAgreement {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub phone: String,
    pub starts_at: i64,
    pub category: String,
    pub ends_at: i64,
    pub visible: bool,
    pub locale: String,
    pub host: String,
    pub rate: f64,
    pub status: EgressAgreementStatus,
}

impl EgressAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressAgreementStatus) -> bool {
        match self.status {
            EgressAgreementStatus::New => {
                match next {
                    EgressAgreementStatus::Running | EgressAgreementStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressAgreementStatus::Queued => {
                return next == EgressAgreementStatus::Done || next == EgressAgreementStatus::Failed;
            }
            EgressAgreementStatus::Running => {
                return next == EgressAgreementStatus::Done || next == EgressAgreementStatus::Failed;
            }
            EgressAgreementStatus::Done => {
                match next {
                    EgressAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: EgressAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress agreement {} cannot move from {} to {}",
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
        format!("{} {}", "egress agreement", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.starts_at as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressAgreement {
        EgressAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            phone: self.phone.clone(),
            starts_at: self.starts_at,
            category: self.category.clone(),
            ends_at: self.ends_at,
            visible: self.visible,
            locale: self.locale.clone(),
            host: self.host.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a egress agreement in status new.
pub fn new_egress_agreement(code: &str, label: &str) -> EgressAgreement {
    EgressAgreement {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        phone: "".to_string(),
        starts_at: 0,
        category: "".to_string(),
        ends_at: 0,
        visible: false,
        locale: "".to_string(),
        host: "".to_string(),
        rate: 0.0,
        status: EgressAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_egress_agreement("code-384", "label-412");
        assert_eq!(item.status, EgressAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_egress_agreement("code-384", "label-412");
        if item.move_to(EgressAgreementStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressAgreementStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_egress_agreement("code-384", "label-412");
        item.status = EgressAgreementStatus::Failed;
        assert!(
            item.move_to(EgressAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
