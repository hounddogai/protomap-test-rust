use super::*;

use acme_kit::text;

/// Where a peering group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PeeringGroupStatus {
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

impl std::fmt::Display for PeeringGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering group as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringGroup {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub priority: i64,
    pub updated_at: i64,
    pub issuer: String,
    pub status: PeeringGroupStatus,
}

impl PeeringGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringGroupStatus) -> bool {
        match self.status {
            PeeringGroupStatus::Planned => {
                match next {
                    PeeringGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringGroupStatus::Scheduled => {
                match next {
                    PeeringGroupStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringGroupStatus::InProgress => {
                match next {
                    PeeringGroupStatus::Completed | PeeringGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringGroupStatus::Completed => {
                return next == PeeringGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.sequence as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringGroup {
        PeeringGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            priority: self.priority,
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering group in status planned.
pub fn new_peering_group(code: &str, title: &str) -> PeeringGroup {
    PeeringGroup {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        priority: 0,
        updated_at: 0,
        issuer: "".to_string(),
        status: PeeringGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_peering_group("code-282", "title-481");
        assert_eq!(item.status, PeeringGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_peering_group("code-282", "title-481");
        if item.move_to(PeeringGroupStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringGroupStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_peering_group("code-282", "title-481");
        item.status = PeeringGroupStatus::Cancelled;
        assert!(
            item.move_to(PeeringGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
