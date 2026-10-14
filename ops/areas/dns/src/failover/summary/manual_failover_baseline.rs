use super::*;

use acme_kit::text;

/// Where a manual failover baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFailoverBaselineStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualFailoverBaselineStatus {
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

impl std::fmt::Display for ManualFailoverBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual failover baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverBaseline {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub ratio: f64,
    pub rate: f64,
    pub rotation_days: i64,
    pub batch_size: i64,
    pub locale: String,
    pub host: String,
    pub timezone: String,
    pub created_at: i64,
    pub status: ManualFailoverBaselineStatus,
}

impl ManualFailoverBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFailoverBaselineStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFailoverBaselineStatus) -> bool {
        match self.status {
            ManualFailoverBaselineStatus::Draft => {
                return next == ManualFailoverBaselineStatus::Closed;
            }
            ManualFailoverBaselineStatus::Pending => {
                return next == ManualFailoverBaselineStatus::Suspended || next == ManualFailoverBaselineStatus::Closed;
            }
            ManualFailoverBaselineStatus::Active => {
                match next {
                    ManualFailoverBaselineStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFailoverBaselineStatus::Suspended => {
                return next == ManualFailoverBaselineStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualFailoverBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual failover baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ratio - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFailoverBaseline {
        ManualFailoverBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            ratio: self.ratio,
            rate: self.rate,
            rotation_days: self.rotation_days,
            batch_size: self.batch_size,
            locale: self.locale.clone(),
            host: self.host.clone(),
            timezone: self.timezone.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual failover baseline in status draft.
pub fn new_manual_failover_baseline(code: &str, label: &str) -> ManualFailoverBaseline {
    ManualFailoverBaseline {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        ratio: 0.0,
        rate: 0.0,
        rotation_days: 0,
        batch_size: 0,
        locale: "".to_string(),
        host: "".to_string(),
        timezone: "".to_string(),
        created_at: 0,
        status: ManualFailoverBaselineStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_failover_baseline("code-122", "label-15");
        assert_eq!(item.status, ManualFailoverBaselineStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_manual_failover_baseline("code-122", "label-15");
        if item.move_to(ManualFailoverBaselineStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFailoverBaselineStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_failover_baseline("code-122", "label-15");
        item.status = ManualFailoverBaselineStatus::Closed;
        assert!(
            item.move_to(ManualFailoverBaselineStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
