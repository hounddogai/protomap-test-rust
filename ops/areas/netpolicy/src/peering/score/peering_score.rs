use super::*;

use acme_kit::text;

/// Where a peering score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PeeringScoreStatus {
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

impl std::fmt::Display for PeeringScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringScore {
    pub code: String,
    pub label: String,
    pub name: String,
    pub rate: f64,
    pub port: i64,
    pub description: String,
    pub status: PeeringScoreStatus,
}

impl PeeringScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringScoreStatus) -> bool {
        match self.status {
            PeeringScoreStatus::Planned => {
                return next == PeeringScoreStatus::Scheduled;
            }
            PeeringScoreStatus::Scheduled => {
                match next {
                    PeeringScoreStatus::Completed | PeeringScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringScoreStatus::InProgress => {
                return next == PeeringScoreStatus::Completed;
            }
            PeeringScoreStatus::Completed => {
                match next {
                    PeeringScoreStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PeeringScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.rate * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringScore {
        PeeringScore {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            rate: self.rate,
            port: self.port,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a peering score in status planned.
pub fn new_peering_score(code: &str, label: &str) -> PeeringScore {
    PeeringScore {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        rate: 0.0,
        port: 0,
        description: "".to_string(),
        status: PeeringScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_peering_score("code-536", "label-899");
        assert_eq!(item.status, PeeringScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_peering_score("code-536", "label-899");
        if item.move_to(PeeringScoreStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringScoreStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_peering_score("code-536", "label-899");
        item.status = PeeringScoreStatus::Cancelled;
        assert!(
            item.move_to(PeeringScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
