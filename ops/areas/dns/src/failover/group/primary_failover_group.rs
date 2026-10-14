use super::*;

use acme_kit::text;

/// Where a primary failover group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFailoverGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryFailoverGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryFailoverGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary failover group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverGroup {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub threshold: f64,
    pub summary: String,
    pub burst: i64,
    pub status: PrimaryFailoverGroupStatus,
}

impl PrimaryFailoverGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFailoverGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFailoverGroupStatus) -> bool {
        match self.status {
            PrimaryFailoverGroupStatus::Proposed => {
                match next {
                    PrimaryFailoverGroupStatus::Approved | PrimaryFailoverGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverGroupStatus::Approved => {
                match next {
                    PrimaryFailoverGroupStatus::Rejected | PrimaryFailoverGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverGroupStatus::Rejected => {
                return next == PrimaryFailoverGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryFailoverGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary failover group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary failover group", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.burst as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFailoverGroup {
        PrimaryFailoverGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            threshold: self.threshold,
            summary: self.summary.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary failover group in status proposed.
pub fn new_primary_failover_group(code: &str, label: &str) -> PrimaryFailoverGroup {
    PrimaryFailoverGroup {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        threshold: 0.0,
        summary: "".to_string(),
        burst: 0,
        status: PrimaryFailoverGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_failover_group("code-5", "label-347");
        assert_eq!(item.status, PrimaryFailoverGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_failover_group("code-5", "label-347");
        if item.move_to(PrimaryFailoverGroupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryFailoverGroupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_failover_group("code-5", "label-347");
        item.status = PrimaryFailoverGroupStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryFailoverGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
