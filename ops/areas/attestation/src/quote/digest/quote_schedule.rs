use super::*;

use acme_kit::text;

/// Where a quote schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for QuoteScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSchedule {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub description: String,
    pub threshold: f64,
    pub contact_email: String,
    pub amount: f64,
    pub phone: String,
    pub category: String,
    pub reviewer: String,
    pub key_id: String,
    pub status: QuoteScheduleStatus,
}

impl QuoteSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteScheduleStatus) -> bool {
        match self.status {
            QuoteScheduleStatus::Idle => {
                return next == QuoteScheduleStatus::Warming;
            }
            QuoteScheduleStatus::Warming => {
                return next == QuoteScheduleStatus::Draining;
            }
            QuoteScheduleStatus::Ready => {
                match next {
                    QuoteScheduleStatus::Draining | QuoteScheduleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteScheduleStatus::Draining => {
                match next {
                    QuoteScheduleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: QuoteScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.threshold + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteSchedule {
        QuoteSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            description: self.description.clone(),
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            phone: self.phone.clone(),
            category: self.category.clone(),
            reviewer: self.reviewer.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a quote schedule in status idle.
pub fn new_quote_schedule(code: &str, name: &str) -> QuoteSchedule {
    QuoteSchedule {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        description: "".to_string(),
        threshold: 0.0,
        contact_email: "".to_string(),
        amount: 0.0,
        phone: "".to_string(),
        category: "".to_string(),
        reviewer: "".to_string(),
        key_id: "".to_string(),
        status: QuoteScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_schedule("code-54", "name-605");
        assert_eq!(item.status, QuoteScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_quote_schedule("code-54", "name-605");
        if item.move_to(QuoteScheduleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteScheduleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_schedule("code-54", "name-605");
        item.status = QuoteScheduleStatus::Stopped;
        assert!(
            item.move_to(QuoteScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
