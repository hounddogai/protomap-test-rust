use super::*;

use acme_kit::text;

/// Where a binding allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BindingAllocationStatus {
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

impl std::fmt::Display for BindingAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding allocation as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAllocation {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub discount: f64,
    pub duration_minutes: i64,
    pub batch_size: i64,
    pub audience: String,
    pub starts_at: i64,
    pub category: String,
    pub pinned: bool,
    pub minimum: f64,
    pub status: BindingAllocationStatus,
}

impl BindingAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingAllocationStatus) -> bool {
        match self.status {
            BindingAllocationStatus::Draft => {
                return next == BindingAllocationStatus::Suspended;
            }
            BindingAllocationStatus::Pending => {
                return next == BindingAllocationStatus::Closed;
            }
            BindingAllocationStatus::Active => {
                return next == BindingAllocationStatus::Suspended || next == BindingAllocationStatus::Closed;
            }
            BindingAllocationStatus::Suspended => {
                match next {
                    BindingAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: BindingAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding allocation {} cannot move from {} to {}",
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
        format!("{} {}", "binding allocation", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingAllocation {
        BindingAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            discount: self.discount,
            duration_minutes: self.duration_minutes,
            batch_size: self.batch_size,
            audience: self.audience.clone(),
            starts_at: self.starts_at,
            category: self.category.clone(),
            pinned: self.pinned,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a binding allocation in status draft.
pub fn new_binding_allocation(code: &str, title: &str) -> BindingAllocation {
    BindingAllocation {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        discount: 0.0,
        duration_minutes: 0,
        batch_size: 0,
        audience: "".to_string(),
        starts_at: 0,
        category: "".to_string(),
        pinned: false,
        minimum: 0.0,
        status: BindingAllocationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_binding_allocation("code-334", "title-715");
        assert_eq!(item.status, BindingAllocationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_binding_allocation("code-334", "title-715");
        if item.move_to(BindingAllocationStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingAllocationStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_binding_allocation("code-334", "title-715");
        item.status = BindingAllocationStatus::Closed;
        assert!(
            item.move_to(BindingAllocationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
