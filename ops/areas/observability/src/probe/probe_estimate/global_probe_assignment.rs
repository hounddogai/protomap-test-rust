use super::*;

use acme_kit::text;

/// Where a global probe assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalProbeAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalProbeAssignmentStatus {
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

impl std::fmt::Display for GlobalProbeAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global probe assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeAssignment {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub audience: String,
    pub port: i64,
    pub currency: String,
    pub quantity: i64,
    pub ceiling: f64,
    pub status: GlobalProbeAssignmentStatus,
}

impl GlobalProbeAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalProbeAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalProbeAssignmentStatus) -> bool {
        match self.status {
            GlobalProbeAssignmentStatus::Draft => {
                return next == GlobalProbeAssignmentStatus::Pending || next == GlobalProbeAssignmentStatus::Closed;
            }
            GlobalProbeAssignmentStatus::Pending => {
                match next {
                    GlobalProbeAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalProbeAssignmentStatus::Active => {
                return next == GlobalProbeAssignmentStatus::Suspended || next == GlobalProbeAssignmentStatus::Closed;
            }
            GlobalProbeAssignmentStatus::Suspended => {
                match next {
                    GlobalProbeAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalProbeAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global probe assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.quantity as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalProbeAssignment {
        GlobalProbeAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            audience: self.audience.clone(),
            port: self.port,
            currency: self.currency.clone(),
            quantity: self.quantity,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global probe assignment in status draft.
pub fn new_global_probe_assignment(code: &str, label: &str) -> GlobalProbeAssignment {
    GlobalProbeAssignment {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        audience: "".to_string(),
        port: 0,
        currency: "".to_string(),
        quantity: 0,
        ceiling: 0.0,
        status: GlobalProbeAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_probe_assignment("code-32", "label-967");
        assert_eq!(item.status, GlobalProbeAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_global_probe_assignment("code-32", "label-967");
        if item.move_to(GlobalProbeAssignmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalProbeAssignmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_probe_assignment("code-32", "label-967");
        item.status = GlobalProbeAssignmentStatus::Closed;
        assert!(
            item.move_to(GlobalProbeAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
