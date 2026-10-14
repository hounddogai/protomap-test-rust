use super::*;

use acme_kit::text;

/// Where a nonce schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceScheduleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl NonceScheduleStatus {
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

impl std::fmt::Display for NonceScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce schedule as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSchedule {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub summary: String,
    pub ends_at: i64,
    pub contact_email: String,
    pub approved: bool,
    pub ceiling: f64,
    pub batch_size: i64,
    pub external_ref: String,
    pub status: NonceScheduleStatus,
}

impl NonceSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceScheduleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceScheduleStatus) -> bool {
        match self.status {
            NonceScheduleStatus::Planned => {
                match next {
                    NonceScheduleStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceScheduleStatus::Scheduled => {
                return next == NonceScheduleStatus::InProgress || next == NonceScheduleStatus::Completed;
            }
            NonceScheduleStatus::InProgress => {
                match next {
                    NonceScheduleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceScheduleStatus::Completed => {
                match next {
                    NonceScheduleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: NonceScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce schedule {} cannot move from {} to {}",
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
        format!("{} {}", "nonce schedule", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.batch_size as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceSchedule {
        NonceSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            summary: self.summary.clone(),
            ends_at: self.ends_at,
            contact_email: self.contact_email.clone(),
            approved: self.approved,
            ceiling: self.ceiling,
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a nonce schedule in status planned.
pub fn new_nonce_schedule(code: &str, label: &str) -> NonceSchedule {
    NonceSchedule {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        summary: "".to_string(),
        ends_at: 0,
        contact_email: "".to_string(),
        approved: false,
        ceiling: 0.0,
        batch_size: 0,
        external_ref: "".to_string(),
        status: NonceScheduleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_nonce_schedule("code-843", "label-804");
        assert_eq!(item.status, NonceScheduleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_nonce_schedule("code-843", "label-804");
        if item.move_to(NonceScheduleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceScheduleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_nonce_schedule("code-843", "label-804");
        item.status = NonceScheduleStatus::Cancelled;
        assert!(
            item.move_to(NonceScheduleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
