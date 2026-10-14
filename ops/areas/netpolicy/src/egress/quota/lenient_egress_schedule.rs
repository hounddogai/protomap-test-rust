use super::*;

use acme_kit::text;

/// Where a lenient egress schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEgressScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientEgressScheduleStatus {
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

impl std::fmt::Display for LenientEgressScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient egress schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressSchedule {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub tenant_id: String,
    pub score: f64,
    pub label: String,
    pub limit: i64,
    pub visible: bool,
    pub port: i64,
    pub approved: bool,
    pub owner_id: String,
    pub status: LenientEgressScheduleStatus,
}

impl LenientEgressSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEgressScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEgressScheduleStatus) -> bool {
        match self.status {
            LenientEgressScheduleStatus::Draft => {
                return next == LenientEgressScheduleStatus::Pending || next == LenientEgressScheduleStatus::Closed;
            }
            LenientEgressScheduleStatus::Pending => {
                match next {
                    LenientEgressScheduleStatus::Suspended | LenientEgressScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressScheduleStatus::Active => {
                return next == LenientEgressScheduleStatus::Suspended || next == LenientEgressScheduleStatus::Closed;
            }
            LenientEgressScheduleStatus::Suspended => {
                match next {
                    LenientEgressScheduleStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientEgressScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient egress schedule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ends_at as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientEgressSchedule {
        LenientEgressSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            label: self.label.clone(),
            limit: self.limit,
            visible: self.visible,
            port: self.port,
            approved: self.approved,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient egress schedule in status draft.
pub fn new_lenient_egress_schedule(code: &str, name: &str) -> LenientEgressSchedule {
    LenientEgressSchedule {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        tenant_id: "".to_string(),
        score: 0.0,
        label: "".to_string(),
        limit: 0,
        visible: false,
        port: 0,
        approved: false,
        owner_id: "".to_string(),
        status: LenientEgressScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_egress_schedule("code-163", "name-749");
        assert_eq!(item.status, LenientEgressScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_egress_schedule("code-163", "name-749");
        if item.move_to(LenientEgressScheduleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEgressScheduleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_egress_schedule("code-163", "name-749");
        item.status = LenientEgressScheduleStatus::Closed;
        assert!(
            item.move_to(LenientEgressScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
