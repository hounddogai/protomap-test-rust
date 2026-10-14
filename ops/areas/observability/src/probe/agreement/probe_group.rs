use super::*;

use acme_kit::text;

/// Where a probe group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ProbeGroupStatus {
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

impl std::fmt::Display for ProbeGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe group as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeGroup {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub archived: bool,
    pub principal: String,
    pub approved: bool,
    pub description: String,
    pub segment: String,
    pub quantity: i64,
    pub status: ProbeGroupStatus,
}

impl ProbeGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeGroupStatus) -> bool {
        match self.status {
            ProbeGroupStatus::Planned => {
                match next {
                    ProbeGroupStatus::Scheduled | ProbeGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeGroupStatus::Scheduled => {
                match next {
                    ProbeGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeGroupStatus::InProgress => {
                return next == ProbeGroupStatus::Completed || next == ProbeGroupStatus::Cancelled;
            }
            ProbeGroupStatus::Completed => {
                return next == ProbeGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ProbeGroup {
        ProbeGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            principal: self.principal.clone(),
            approved: self.approved,
            description: self.description.clone(),
            segment: self.segment.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a probe group in status planned.
pub fn new_probe_group(code: &str, label: &str) -> ProbeGroup {
    ProbeGroup {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        archived: false,
        principal: "".to_string(),
        approved: false,
        description: "".to_string(),
        segment: "".to_string(),
        quantity: 0,
        status: ProbeGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_probe_group("code-452", "label-161");
        assert_eq!(item.status, ProbeGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_probe_group("code-452", "label-161");
        if item.move_to(ProbeGroupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeGroupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_probe_group("code-452", "label-161");
        item.status = ProbeGroupStatus::Cancelled;
        assert!(
            item.move_to(ProbeGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
