use super::*;

use acme_kit::text;

/// Where a adaptive plan entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePlanEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptivePlanEntryStatus {
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

impl std::fmt::Display for AdaptivePlanEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive plan entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanEntry {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub owner_id: String,
    pub issuer: String,
    pub reviewer: String,
    pub key_id: String,
    pub limit: i64,
    pub segment: String,
    pub unit: String,
    pub status: AdaptivePlanEntryStatus,
}

impl AdaptivePlanEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePlanEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePlanEntryStatus) -> bool {
        match self.status {
            AdaptivePlanEntryStatus::Planned => {
                return next == AdaptivePlanEntryStatus::Scheduled;
            }
            AdaptivePlanEntryStatus::Scheduled => {
                match next {
                    AdaptivePlanEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePlanEntryStatus::InProgress => {
                match next {
                    AdaptivePlanEntryStatus::Completed | AdaptivePlanEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePlanEntryStatus::Completed => {
                return next == AdaptivePlanEntryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptivePlanEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive plan entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive plan entry", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePlanEntry {
        AdaptivePlanEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            reviewer: self.reviewer.clone(),
            key_id: self.key_id.clone(),
            limit: self.limit,
            segment: self.segment.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive plan entry in status planned.
pub fn new_adaptive_plan_entry(code: &str, name: &str) -> AdaptivePlanEntry {
    AdaptivePlanEntry {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        owner_id: "".to_string(),
        issuer: "".to_string(),
        reviewer: "".to_string(),
        key_id: "".to_string(),
        limit: 0,
        segment: "".to_string(),
        unit: "".to_string(),
        status: AdaptivePlanEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_plan_entry("code-877", "name-73");
        assert_eq!(item.status, AdaptivePlanEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_adaptive_plan_entry("code-877", "name-73");
        if item.move_to(AdaptivePlanEntryStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePlanEntryStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_plan_entry("code-877", "name-73");
        item.status = AdaptivePlanEntryStatus::Cancelled;
        assert!(
            item.move_to(AdaptivePlanEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
