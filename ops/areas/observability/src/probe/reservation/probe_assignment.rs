use super::*;

use acme_kit::text;

/// Where a probe assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ProbeAssignmentStatus {
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

impl std::fmt::Display for ProbeAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAssignment {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub window_days: i64,
    pub country: String,
    pub unit: String,
    pub limit: i64,
    pub expires_at: i64,
    pub margin: f64,
    pub locale: String,
    pub status: ProbeAssignmentStatus,
}

impl ProbeAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeAssignmentStatus) -> bool {
        match self.status {
            ProbeAssignmentStatus::Open => {
                return next == ProbeAssignmentStatus::InReview || next == ProbeAssignmentStatus::Reopened;
            }
            ProbeAssignmentStatus::InReview => {
                return next == ProbeAssignmentStatus::Resolved || next == ProbeAssignmentStatus::Archived;
            }
            ProbeAssignmentStatus::Resolved => {
                match next {
                    ProbeAssignmentStatus::Reopened | ProbeAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeAssignmentStatus::Reopened => {
                return next == ProbeAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe assignment", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.window_days as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeAssignment {
        ProbeAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            window_days: self.window_days,
            country: self.country.clone(),
            unit: self.unit.clone(),
            limit: self.limit,
            expires_at: self.expires_at,
            margin: self.margin,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a probe assignment in status open.
pub fn new_probe_assignment(code: &str, name: &str) -> ProbeAssignment {
    ProbeAssignment {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        window_days: 0,
        country: "".to_string(),
        unit: "".to_string(),
        limit: 0,
        expires_at: 0,
        margin: 0.0,
        locale: "".to_string(),
        status: ProbeAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_probe_assignment("code-224", "name-765");
        assert_eq!(item.status, ProbeAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_probe_assignment("code-224", "name-765");
        if item.move_to(ProbeAssignmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeAssignmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_probe_assignment("code-224", "name-765");
        item.status = ProbeAssignmentStatus::Archived;
        assert!(
            item.move_to(ProbeAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
