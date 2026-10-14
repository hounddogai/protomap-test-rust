use super::*;

use acme_kit::text;

/// Where a peering window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PeeringWindowStatus {
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

impl std::fmt::Display for PeeringWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringWindow {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub key_id: String,
    pub maximum: f64,
    pub duration_minutes: i64,
    pub cost: f64,
    pub timezone: String,
    pub principal: String,
    pub status: PeeringWindowStatus,
}

impl PeeringWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringWindowStatus) -> bool {
        match self.status {
            PeeringWindowStatus::Open => {
                match next {
                    PeeringWindowStatus::Resolved | PeeringWindowStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringWindowStatus::InReview => {
                return next == PeeringWindowStatus::Resolved || next == PeeringWindowStatus::Archived;
            }
            PeeringWindowStatus::Resolved => {
                return next == PeeringWindowStatus::Reopened || next == PeeringWindowStatus::Archived;
            }
            PeeringWindowStatus::Reopened => {
                return next == PeeringWindowStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering window", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.maximum * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringWindow {
        PeeringWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            duration_minutes: self.duration_minutes,
            cost: self.cost,
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering window in status open.
pub fn new_peering_window(code: &str, title: &str) -> PeeringWindow {
    PeeringWindow {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        key_id: "".to_string(),
        maximum: 0.0,
        duration_minutes: 0,
        cost: 0.0,
        timezone: "".to_string(),
        principal: "".to_string(),
        status: PeeringWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_peering_window("code-166", "title-938");
        assert_eq!(item.status, PeeringWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_peering_window("code-166", "title-938");
        if item.move_to(PeeringWindowStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringWindowStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_peering_window("code-166", "title-938");
        item.status = PeeringWindowStatus::Archived;
        assert!(
            item.move_to(PeeringWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
