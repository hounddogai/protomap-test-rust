use super::*;

use acme_kit::text;

/// Where a escalation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EscalationScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EscalationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationScore {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub visible: bool,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub cluster: String,
    pub score: f64,
    pub burst: i64,
    pub budget: f64,
    pub status: EscalationScoreStatus,
}

impl EscalationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationScoreStatus) -> bool {
        match self.status {
            EscalationScoreStatus::Requested => {
                return next == EscalationScoreStatus::Expired;
            }
            EscalationScoreStatus::Granted => {
                match next {
                    EscalationScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationScoreStatus::Revoked => {
                return next == EscalationScoreStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation score", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationScore {
        EscalationScore {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            visible: self.visible,
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            cluster: self.cluster.clone(),
            score: self.score,
            burst: self.burst,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a escalation score in status requested.
pub fn new_escalation_score(code: &str, name: &str) -> EscalationScore {
    EscalationScore {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        visible: false,
        ttl_seconds: 0,
        attempts: 0,
        cluster: "".to_string(),
        score: 0.0,
        burst: 0,
        budget: 0.0,
        status: EscalationScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_escalation_score("code-869", "name-577");
        assert_eq!(item.status, EscalationScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_escalation_score("code-869", "name-577");
        if item.move_to(EscalationScoreStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationScoreStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_escalation_score("code-869", "name-577");
        item.status = EscalationScoreStatus::Expired;
        assert!(
            item.move_to(EscalationScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
