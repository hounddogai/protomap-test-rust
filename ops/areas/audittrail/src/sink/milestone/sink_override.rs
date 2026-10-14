use super::*;

use acme_kit::text;

/// Where a sink override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SinkOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SinkOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink override as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverride {
    pub code: String,
    pub name: String,
    pub description: String,
    pub cost: f64,
    pub attempts: i64,
    pub status: SinkOverrideStatus,
}

impl SinkOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkOverrideStatus) -> bool {
        match self.status {
            SinkOverrideStatus::Proposed => {
                match next {
                    SinkOverrideStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkOverrideStatus::Approved => {
                return next == SinkOverrideStatus::Rejected || next == SinkOverrideStatus::Withdrawn;
            }
            SinkOverrideStatus::Rejected => {
                return next == SinkOverrideStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.attempts as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkOverride {
        SinkOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            cost: self.cost,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a sink override in status proposed.
pub fn new_sink_override(code: &str, name: &str) -> SinkOverride {
    SinkOverride {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        cost: 0.0,
        attempts: 0,
        status: SinkOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_sink_override("code-415", "name-979");
        assert_eq!(item.status, SinkOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_sink_override("code-415", "name-979");
        if item.move_to(SinkOverrideStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkOverrideStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_sink_override("code-415", "name-979");
        item.status = SinkOverrideStatus::Withdrawn;
        assert!(
            item.move_to(SinkOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
