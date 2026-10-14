use super::*;

use acme_kit::text;

/// Where a manual isolation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualIsolationTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualIsolationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualIsolationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual isolation target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationTarget {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub channel: String,
    pub starts_at: i64,
    pub weight: f64,
    pub visible: bool,
    pub status: ManualIsolationTargetStatus,
}

impl ManualIsolationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualIsolationTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualIsolationTargetStatus) -> bool {
        match self.status {
            ManualIsolationTargetStatus::Proposed => {
                match next {
                    ManualIsolationTargetStatus::Approved | ManualIsolationTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationTargetStatus::Approved => {
                match next {
                    ManualIsolationTargetStatus::Rejected | ManualIsolationTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIsolationTargetStatus::Rejected => {
                match next {
                    ManualIsolationTargetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualIsolationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual isolation target {} cannot move from {} to {}",
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
        format!("{} {}", "manual isolation target", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.starts_at as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualIsolationTarget {
        ManualIsolationTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            channel: self.channel.clone(),
            starts_at: self.starts_at,
            weight: self.weight,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual isolation target in status proposed.
pub fn new_manual_isolation_target(code: &str, label: &str) -> ManualIsolationTarget {
    ManualIsolationTarget {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        channel: "".to_string(),
        starts_at: 0,
        weight: 0.0,
        visible: false,
        status: ManualIsolationTargetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_isolation_target("code-734", "label-933");
        assert_eq!(item.status, ManualIsolationTargetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_manual_isolation_target("code-734", "label-933");
        if item.move_to(ManualIsolationTargetStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualIsolationTargetStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_isolation_target("code-734", "label-933");
        item.status = ManualIsolationTargetStatus::Withdrawn;
        assert!(
            item.move_to(ManualIsolationTargetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
