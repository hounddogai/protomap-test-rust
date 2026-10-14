use super::*;

use acme_kit::text;

/// Where a timeline plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelinePlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TimelinePlanStatus {
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

impl std::fmt::Display for TimelinePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlan {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub minimum: f64,
    pub owner_id: String,
    pub key_id: String,
    pub region: String,
    pub status: TimelinePlanStatus,
}

impl TimelinePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelinePlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelinePlanStatus) -> bool {
        match self.status {
            TimelinePlanStatus::Draft => {
                return next == TimelinePlanStatus::Active || next == TimelinePlanStatus::Suspended;
            }
            TimelinePlanStatus::Pending => {
                match next {
                    TimelinePlanStatus::Active | TimelinePlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelinePlanStatus::Active => {
                match next {
                    TimelinePlanStatus::Suspended | TimelinePlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelinePlanStatus::Suspended => {
                return next == TimelinePlanStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelinePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.minimum + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelinePlan {
        TimelinePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            minimum: self.minimum,
            owner_id: self.owner_id.clone(),
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a timeline plan in status draft.
pub fn new_timeline_plan(code: &str, name: &str) -> TimelinePlan {
    TimelinePlan {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        minimum: 0.0,
        owner_id: "".to_string(),
        key_id: "".to_string(),
        region: "".to_string(),
        status: TimelinePlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_timeline_plan("code-189", "name-854");
        assert_eq!(item.status, TimelinePlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_timeline_plan("code-189", "name-854");
        if item.move_to(TimelinePlanStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelinePlanStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_timeline_plan("code-189", "name-854");
        item.status = TimelinePlanStatus::Closed;
        assert!(
            item.move_to(TimelinePlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
