use super::*;

use acme_kit::text;

/// Where a plan entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanEntry {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub capacity: i64,
    pub grace_days: i64,
    pub retries: i64,
    pub starts_at: i64,
    pub status: PlanEntryStatus,
}

impl PlanEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanEntryStatus) -> bool {
        match self.status {
            PlanEntryStatus::Proposed => {
                match next {
                    PlanEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanEntryStatus::Approved => {
                match next {
                    PlanEntryStatus::Rejected | PlanEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanEntryStatus::Rejected => {
                match next {
                    PlanEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PlanEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan entry {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.grace_days as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanEntry {
        PlanEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            capacity: self.capacity,
            grace_days: self.grace_days,
            retries: self.retries,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a plan entry in status proposed.
pub fn new_plan_entry(code: &str, title: &str) -> PlanEntry {
    PlanEntry {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        capacity: 0,
        grace_days: 0,
        retries: 0,
        starts_at: 0,
        status: PlanEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_plan_entry("code-512", "title-626");
        assert_eq!(item.status, PlanEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_plan_entry("code-512", "title-626");
        if item.move_to(PlanEntryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanEntryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_plan_entry("code-512", "title-626");
        item.status = PlanEntryStatus::Withdrawn;
        assert!(
            item.move_to(PlanEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
