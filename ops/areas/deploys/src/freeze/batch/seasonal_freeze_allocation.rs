use super::*;

use acme_kit::text;

/// Where a seasonal freeze allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFreezeAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalFreezeAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for SeasonalFreezeAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal freeze allocation as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeAllocation {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub priority: i64,
    pub phone: String,
    pub status: SeasonalFreezeAllocationStatus,
}

impl SeasonalFreezeAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFreezeAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFreezeAllocationStatus) -> bool {
        match self.status {
            SeasonalFreezeAllocationStatus::Draft => {
                match next {
                    SeasonalFreezeAllocationStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFreezeAllocationStatus::Pending => {
                match next {
                    SeasonalFreezeAllocationStatus::Active | SeasonalFreezeAllocationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFreezeAllocationStatus::Active => {
                return next == SeasonalFreezeAllocationStatus::Suspended
                    || next == SeasonalFreezeAllocationStatus::Closed;
            }
            SeasonalFreezeAllocationStatus::Suspended => {
                match next {
                    SeasonalFreezeAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: SeasonalFreezeAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal freeze allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> SeasonalFreezeAllocation {
        SeasonalFreezeAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            priority: self.priority,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal freeze allocation in status draft.
pub fn new_seasonal_freeze_allocation(code: &str, title: &str) -> SeasonalFreezeAllocation {
    SeasonalFreezeAllocation {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        priority: 0,
        phone: "".to_string(),
        status: SeasonalFreezeAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_seasonal_freeze_allocation("code-118", "title-255");
        assert_eq!(item.status, SeasonalFreezeAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_seasonal_freeze_allocation("code-118", "title-255");
        if item.move_to(SeasonalFreezeAllocationStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFreezeAllocationStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_seasonal_freeze_allocation("code-118", "title-255");
        item.status = SeasonalFreezeAllocationStatus::Closed;
        assert!(
            item.move_to(SeasonalFreezeAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
