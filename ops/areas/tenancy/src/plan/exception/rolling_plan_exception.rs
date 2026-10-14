use super::*;

use acme_kit::text;

/// Where a rolling plan exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPlanExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingPlanExceptionStatus {
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

impl std::fmt::Display for RollingPlanExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling plan exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanException {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub ceiling: f64,
    pub sequence: i64,
    pub updated_at: i64,
    pub currency: String,
    pub name: String,
    pub status: RollingPlanExceptionStatus,
}

impl RollingPlanException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPlanExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPlanExceptionStatus) -> bool {
        match self.status {
            RollingPlanExceptionStatus::Draft => {
                match next {
                    RollingPlanExceptionStatus::Pending | RollingPlanExceptionStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPlanExceptionStatus::Pending => {
                return next == RollingPlanExceptionStatus::Suspended;
            }
            RollingPlanExceptionStatus::Active => {
                return next == RollingPlanExceptionStatus::Suspended;
            }
            RollingPlanExceptionStatus::Suspended => {
                match next {
                    RollingPlanExceptionStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingPlanExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling plan exception {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.sequence as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPlanException {
        RollingPlanException {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            ceiling: self.ceiling,
            sequence: self.sequence,
            updated_at: self.updated_at,
            currency: self.currency.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling plan exception in status draft.
pub fn new_rolling_plan_exception(code: &str, title: &str) -> RollingPlanException {
    RollingPlanException {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        ceiling: 0.0,
        sequence: 0,
        updated_at: 0,
        currency: "".to_string(),
        name: "".to_string(),
        status: RollingPlanExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_plan_exception("code-421", "title-216");
        assert_eq!(item.status, RollingPlanExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rolling_plan_exception("code-421", "title-216");
        if item.move_to(RollingPlanExceptionStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPlanExceptionStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_plan_exception("code-421", "title-216");
        item.status = RollingPlanExceptionStatus::Closed;
        assert!(
            item.move_to(RollingPlanExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
