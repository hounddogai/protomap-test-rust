use super::*;

use acme_kit::text;

/// Where a federation quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FederationQuotaStatus {
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

impl std::fmt::Display for FederationQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationQuota {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub duration_minutes: i64,
    pub visible: bool,
    pub weight: f64,
    pub capacity: i64,
    pub ceiling: f64,
    pub approved: bool,
    pub cipher: String,
    pub status: FederationQuotaStatus,
}

impl FederationQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationQuotaStatus) -> bool {
        match self.status {
            FederationQuotaStatus::Planned => {
                match next {
                    FederationQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationQuotaStatus::Scheduled => {
                return next == FederationQuotaStatus::Completed;
            }
            FederationQuotaStatus::InProgress => {
                return next == FederationQuotaStatus::Completed || next == FederationQuotaStatus::Cancelled;
            }
            FederationQuotaStatus::Completed => {
                return next == FederationQuotaStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "federation quota", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_volume(&self) -> f64 {
        (self.weight * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationQuota {
        FederationQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            weight: self.weight,
            capacity: self.capacity,
            ceiling: self.ceiling,
            approved: self.approved,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a federation quota in status planned.
pub fn new_federation_quota(code: &str, name: &str) -> FederationQuota {
    FederationQuota {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        duration_minutes: 0,
        visible: false,
        weight: 0.0,
        capacity: 0,
        ceiling: 0.0,
        approved: false,
        cipher: "".to_string(),
        status: FederationQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_federation_quota("code-586", "name-382");
        assert_eq!(item.status, FederationQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_federation_quota("code-586", "name-382");
        if item.move_to(FederationQuotaStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationQuotaStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_federation_quota("code-586", "name-382");
        item.status = FederationQuotaStatus::Cancelled;
        assert!(
            item.move_to(FederationQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
