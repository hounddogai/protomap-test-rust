use super::*;

use acme_kit::text;

/// Where a freeze group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FreezeGroupStatus {
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

impl std::fmt::Display for FreezeGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze group as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeGroup {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub cost: f64,
    pub updated_at: i64,
    pub active: bool,
    pub description: String,
    pub tags: Vec<String>,
    pub audience: String,
    pub region: String,
    pub summary: String,
    pub status: FreezeGroupStatus,
}

impl FreezeGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeGroupStatus) -> bool {
        match self.status {
            FreezeGroupStatus::Planned => {
                return next == FreezeGroupStatus::Cancelled;
            }
            FreezeGroupStatus::Scheduled => {
                match next {
                    FreezeGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeGroupStatus::InProgress => {
                match next {
                    FreezeGroupStatus::Completed | FreezeGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeGroupStatus::Completed => {
                return next == FreezeGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze group {} cannot move from {} to {}",
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
        format!("{} {}", "freeze group", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.updated_at as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeGroup {
        FreezeGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            cost: self.cost,
            updated_at: self.updated_at,
            active: self.active,
            description: self.description.clone(),
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            region: self.region.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a freeze group in status planned.
pub fn new_freeze_group(code: &str, title: &str) -> FreezeGroup {
    FreezeGroup {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        cost: 0.0,
        updated_at: 0,
        active: false,
        description: "".to_string(),
        tags: Vec::new(),
        audience: "".to_string(),
        region: "".to_string(),
        summary: "".to_string(),
        status: FreezeGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_freeze_group("code-467", "title-510");
        assert_eq!(item.status, FreezeGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_freeze_group("code-467", "title-510");
        if item.move_to(FreezeGroupStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeGroupStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_freeze_group("code-467", "title-510");
        item.status = FreezeGroupStatus::Cancelled;
        assert!(
            item.move_to(FreezeGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
