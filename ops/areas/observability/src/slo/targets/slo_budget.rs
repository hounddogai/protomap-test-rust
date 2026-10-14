use super::*;

use acme_kit::text;

/// Where a slo budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SloBudgetStatus {
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

impl std::fmt::Display for SloBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a slo budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SloBudget {
    pub code: String,
    pub title: String,
    pub host: String,
    pub quota_units: i64,
    pub locale: String,
    pub port: i64,
    pub starts_at: i64,
    pub fingerprint: String,
    pub floor_value: f64,
    pub discount: f64,
    pub burst: i64,
    pub status: SloBudgetStatus,
}

impl SloBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloBudgetStatus) -> bool {
        match self.status {
            SloBudgetStatus::Draft => {
                return next == SloBudgetStatus::Pending || next == SloBudgetStatus::Suspended;
            }
            SloBudgetStatus::Pending => {
                match next {
                    SloBudgetStatus::Active | SloBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloBudgetStatus::Active => {
                return next == SloBudgetStatus::Suspended || next == SloBudgetStatus::Closed;
            }
            SloBudgetStatus::Suspended => {
                return next == SloBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo budget {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quota_units as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloBudget {
        SloBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            quota_units: self.quota_units,
            locale: self.locale.clone(),
            port: self.port,
            starts_at: self.starts_at,
            fingerprint: self.fingerprint.clone(),
            floor_value: self.floor_value,
            discount: self.discount,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a slo budget in status draft.
pub fn new_slo_budget(code: &str, title: &str) -> SloBudget {
    SloBudget {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        quota_units: 0,
        locale: "".to_string(),
        port: 0,
        starts_at: 0,
        fingerprint: "".to_string(),
        floor_value: 0.0,
        discount: 0.0,
        burst: 0,
        status: SloBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_slo_budget("code-578", "title-623");
        assert_eq!(item.status, SloBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_slo_budget("code-578", "title-623");
        if item.move_to(SloBudgetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloBudgetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_slo_budget("code-578", "title-623");
        item.status = SloBudgetStatus::Closed;
        assert!(item.move_to(SloBudgetStatus::Draft).is_err(), "expected move to fail");
    }
}
