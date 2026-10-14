use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a slo allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloAllocationStatus {
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

impl std::fmt::Display for SloAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo allocation as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloAllocation {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub attributes: BTreeMap<String, String>,
    pub burst: i64,
    pub locale: String,
    pub quota_units: i64,
    pub status: SloAllocationStatus,
}

impl SloAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloAllocationStatus) -> bool {
        match self.status {
            SloAllocationStatus::Planned => {
                return next == SloAllocationStatus::Cancelled;
            }
            SloAllocationStatus::Scheduled => {
                return next == SloAllocationStatus::InProgress;
            }
            SloAllocationStatus::InProgress => {
                return next == SloAllocationStatus::Completed;
            }
            SloAllocationStatus::Completed => {
                match next {
                    SloAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SloAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloAllocation {
        SloAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            attributes: self.attributes.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a slo allocation in status planned.
pub fn new_slo_allocation(code: &str, title: &str) -> SloAllocation {
    SloAllocation {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        attributes: BTreeMap::new(),
        burst: 0,
        locale: "".to_string(),
        quota_units: 0,
        status: SloAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_slo_allocation("code-802", "title-943");
        assert_eq!(item.status, SloAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_slo_allocation("code-802", "title-943");
        if item.move_to(SloAllocationStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloAllocationStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_slo_allocation("code-802", "title-943");
        item.status = SloAllocationStatus::Cancelled;
        assert!(
            item.move_to(SloAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
