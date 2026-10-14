use super::*;

use acme_kit::text;

/// Where a lenient failover window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFailoverWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientFailoverWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientFailoverWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient failover window as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFailoverWindow {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub active: bool,
    pub reviewer: String,
    pub ceiling: f64,
    pub minimum: f64,
    pub amount: f64,
    pub grace_days: i64,
    pub updated_at: i64,
    pub revision: i64,
    pub status: LenientFailoverWindowStatus,
}

impl LenientFailoverWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFailoverWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFailoverWindowStatus) -> bool {
        match self.status {
            LenientFailoverWindowStatus::Created => {
                return next == LenientFailoverWindowStatus::Published;
            }
            LenientFailoverWindowStatus::Validated => {
                return next == LenientFailoverWindowStatus::Published || next == LenientFailoverWindowStatus::Retired;
            }
            LenientFailoverWindowStatus::Published => {
                return next == LenientFailoverWindowStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientFailoverWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient failover window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient failover window", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.minimum + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFailoverWindow {
        LenientFailoverWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            active: self.active,
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            minimum: self.minimum,
            amount: self.amount,
            grace_days: self.grace_days,
            updated_at: self.updated_at,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient failover window in status created.
pub fn new_lenient_failover_window(code: &str, title: &str) -> LenientFailoverWindow {
    LenientFailoverWindow {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        active: false,
        reviewer: "".to_string(),
        ceiling: 0.0,
        minimum: 0.0,
        amount: 0.0,
        grace_days: 0,
        updated_at: 0,
        revision: 0,
        status: LenientFailoverWindowStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_failover_window("code-374", "title-71");
        assert_eq!(item.status, LenientFailoverWindowStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_lenient_failover_window("code-374", "title-71");
        if item.move_to(LenientFailoverWindowStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFailoverWindowStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_failover_window("code-374", "title-71");
        item.status = LenientFailoverWindowStatus::Retired;
        assert!(
            item.move_to(LenientFailoverWindowStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
