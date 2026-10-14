use super::*;

use acme_kit::text;

/// Where a binding schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BindingScheduleStatus {
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

impl std::fmt::Display for BindingScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSchedule {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub score: f64,
    pub tags: Vec<String>,
    pub channel: String,
    pub phone: String,
    pub unit: String,
    pub namespace: String,
    pub minimum: f64,
    pub status: BindingScheduleStatus,
}

impl BindingSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingScheduleStatus) -> bool {
        match self.status {
            BindingScheduleStatus::Draft => {
                return next == BindingScheduleStatus::Active || next == BindingScheduleStatus::Suspended;
            }
            BindingScheduleStatus::Pending => {
                match next {
                    BindingScheduleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingScheduleStatus::Active => {
                return next == BindingScheduleStatus::Suspended;
            }
            BindingScheduleStatus::Suspended => {
                return next == BindingScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.score + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingSchedule {
        BindingSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            score: self.score,
            tags: self.tags.clone(),
            channel: self.channel.clone(),
            phone: self.phone.clone(),
            unit: self.unit.clone(),
            namespace: self.namespace.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a binding schedule in status draft.
pub fn new_binding_schedule(code: &str, title: &str) -> BindingSchedule {
    BindingSchedule {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        score: 0.0,
        tags: Vec::new(),
        channel: "".to_string(),
        phone: "".to_string(),
        unit: "".to_string(),
        namespace: "".to_string(),
        minimum: 0.0,
        status: BindingScheduleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_binding_schedule("code-468", "title-494");
        assert_eq!(item.status, BindingScheduleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_binding_schedule("code-468", "title-494");
        if item.move_to(BindingScheduleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingScheduleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_binding_schedule("code-468", "title-494");
        item.status = BindingScheduleStatus::Closed;
        assert!(
            item.move_to(BindingScheduleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
