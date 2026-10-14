use super::*;

use acme_kit::text;

/// Where a egress score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EgressScoreStatus {
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

impl std::fmt::Display for EgressScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressScore {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub archived: bool,
    pub floor_value: f64,
    pub external_ref: String,
    pub name: String,
    pub issuer: String,
    pub locale: String,
    pub principal: String,
    pub status: EgressScoreStatus,
}

impl EgressScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressScoreStatus) -> bool {
        match self.status {
            EgressScoreStatus::Planned => {
                match next {
                    EgressScoreStatus::InProgress | EgressScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressScoreStatus::Scheduled => {
                return next == EgressScoreStatus::Cancelled;
            }
            EgressScoreStatus::InProgress => {
                match next {
                    EgressScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressScoreStatus::Completed => {
                return next == EgressScoreStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress score", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> EgressScore {
        EgressScore {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            archived: self.archived,
            floor_value: self.floor_value,
            external_ref: self.external_ref.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            locale: self.locale.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a egress score in status planned.
pub fn new_egress_score(code: &str, label: &str) -> EgressScore {
    EgressScore {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        archived: false,
        floor_value: 0.0,
        external_ref: "".to_string(),
        name: "".to_string(),
        issuer: "".to_string(),
        locale: "".to_string(),
        principal: "".to_string(),
        status: EgressScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_egress_score("code-504", "label-313");
        assert_eq!(item.status, EgressScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_egress_score("code-504", "label-313");
        if item.move_to(EgressScoreStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressScoreStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_egress_score("code-504", "label-313");
        item.status = EgressScoreStatus::Cancelled;
        assert!(
            item.move_to(EgressScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
