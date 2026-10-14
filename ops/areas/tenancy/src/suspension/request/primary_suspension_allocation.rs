use super::*;

use acme_kit::text;

/// Where a primary suspension allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySuspensionAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimarySuspensionAllocationStatus {
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

impl std::fmt::Display for PrimarySuspensionAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary suspension allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionAllocation {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub rate: f64,
    pub tags: Vec<String>,
    pub weight: f64,
    pub currency: String,
    pub batch_size: i64,
    pub updated_at: i64,
    pub notes: String,
    pub unit: String,
    pub status: PrimarySuspensionAllocationStatus,
}

impl PrimarySuspensionAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySuspensionAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySuspensionAllocationStatus) -> bool {
        match self.status {
            PrimarySuspensionAllocationStatus::Planned => {
                match next {
                    PrimarySuspensionAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySuspensionAllocationStatus::Scheduled => {
                return next == PrimarySuspensionAllocationStatus::Completed
                    || next == PrimarySuspensionAllocationStatus::Cancelled;
            }
            PrimarySuspensionAllocationStatus::InProgress => {
                return next == PrimarySuspensionAllocationStatus::Completed
                    || next == PrimarySuspensionAllocationStatus::Cancelled;
            }
            PrimarySuspensionAllocationStatus::Completed => {
                return next == PrimarySuspensionAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySuspensionAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary suspension allocation {} cannot move from {} to {}",
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
        format!("{} {}", "primary suspension allocation", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.batch_size as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySuspensionAllocation {
        PrimarySuspensionAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            rate: self.rate,
            tags: self.tags.clone(),
            weight: self.weight,
            currency: self.currency.clone(),
            batch_size: self.batch_size,
            updated_at: self.updated_at,
            notes: self.notes.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary suspension allocation in status planned.
pub fn new_primary_suspension_allocation(code: &str, title: &str) -> PrimarySuspensionAllocation {
    PrimarySuspensionAllocation {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        rate: 0.0,
        tags: Vec::new(),
        weight: 0.0,
        currency: "".to_string(),
        batch_size: 0,
        updated_at: 0,
        notes: "".to_string(),
        unit: "".to_string(),
        status: PrimarySuspensionAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_suspension_allocation("code-313", "title-400");
        assert_eq!(item.status, PrimarySuspensionAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_primary_suspension_allocation("code-313", "title-400");
        if item.move_to(PrimarySuspensionAllocationStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySuspensionAllocationStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_suspension_allocation("code-313", "title-400");
        item.status = PrimarySuspensionAllocationStatus::Cancelled;
        assert!(
            item.move_to(PrimarySuspensionAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
