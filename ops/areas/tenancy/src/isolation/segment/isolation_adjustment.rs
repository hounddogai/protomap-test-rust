use super::*;

use acme_kit::text;

/// Where a isolation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IsolationAdjustmentStatus {
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

impl std::fmt::Display for IsolationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation adjustment as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationAdjustment {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub external_ref: String,
    pub priority: i64,
    pub approved: bool,
    pub host: String,
    pub status: IsolationAdjustmentStatus,
}

impl IsolationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationAdjustmentStatus) -> bool {
        match self.status {
            IsolationAdjustmentStatus::Draft => {
                match next {
                    IsolationAdjustmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationAdjustmentStatus::Pending => {
                return next == IsolationAdjustmentStatus::Active || next == IsolationAdjustmentStatus::Suspended;
            }
            IsolationAdjustmentStatus::Active => {
                return next == IsolationAdjustmentStatus::Closed;
            }
            IsolationAdjustmentStatus::Suspended => {
                return next == IsolationAdjustmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> IsolationAdjustment {
        IsolationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            priority: self.priority,
            approved: self.approved,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a isolation adjustment in status draft.
pub fn new_isolation_adjustment(code: &str, title: &str) -> IsolationAdjustment {
    IsolationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        external_ref: "".to_string(),
        priority: 0,
        approved: false,
        host: "".to_string(),
        status: IsolationAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_isolation_adjustment("code-210", "title-754");
        assert_eq!(item.status, IsolationAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_isolation_adjustment("code-210", "title-754");
        if item.move_to(IsolationAdjustmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationAdjustmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_isolation_adjustment("code-210", "title-754");
        item.status = IsolationAdjustmentStatus::Closed;
        assert!(
            item.move_to(IsolationAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
