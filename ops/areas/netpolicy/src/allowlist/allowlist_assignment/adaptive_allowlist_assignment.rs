use super::*;

use acme_kit::text;

/// Where a adaptive allowlist assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAllowlistAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveAllowlistAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AdaptiveAllowlistAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive allowlist assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistAssignment {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub amount: f64,
    pub external_ref: String,
    pub margin: f64,
    pub port: i64,
    pub host: String,
    pub status: AdaptiveAllowlistAssignmentStatus,
}

impl AdaptiveAllowlistAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAllowlistAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAllowlistAssignmentStatus) -> bool {
        match self.status {
            AdaptiveAllowlistAssignmentStatus::Draft => {
                match next {
                    AdaptiveAllowlistAssignmentStatus::Active | AdaptiveAllowlistAssignmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistAssignmentStatus::Pending => {
                match next {
                    AdaptiveAllowlistAssignmentStatus::Suspended | AdaptiveAllowlistAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistAssignmentStatus::Active => {
                match next {
                    AdaptiveAllowlistAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveAllowlistAssignmentStatus::Suspended => {
                match next {
                    AdaptiveAllowlistAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveAllowlistAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive allowlist assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.amount + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAllowlistAssignment {
        AdaptiveAllowlistAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            external_ref: self.external_ref.clone(),
            margin: self.margin,
            port: self.port,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive allowlist assignment in status draft.
pub fn new_adaptive_allowlist_assignment(code: &str, name: &str) -> AdaptiveAllowlistAssignment {
    AdaptiveAllowlistAssignment {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        amount: 0.0,
        external_ref: "".to_string(),
        margin: 0.0,
        port: 0,
        host: "".to_string(),
        status: AdaptiveAllowlistAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_allowlist_assignment("code-959", "name-679");
        assert_eq!(item.status, AdaptiveAllowlistAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_adaptive_allowlist_assignment("code-959", "name-679");
        if item.move_to(AdaptiveAllowlistAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveAllowlistAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_allowlist_assignment("code-959", "name-679");
        item.status = AdaptiveAllowlistAssignmentStatus::Closed;
        assert!(
            item.move_to(AdaptiveAllowlistAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
