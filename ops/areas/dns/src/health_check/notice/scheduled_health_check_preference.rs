use super::*;

use acme_kit::text;

/// Where a scheduled health check preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledHealthCheckPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledHealthCheckPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ScheduledHealthCheckPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled health check preference as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckPreference {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub scope: String,
    pub fingerprint: String,
    pub starts_at: i64,
    pub external_ref: String,
    pub region: String,
    pub namespace: String,
    pub weight: f64,
    pub revision: i64,
    pub status: ScheduledHealthCheckPreferenceStatus,
}

impl ScheduledHealthCheckPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledHealthCheckPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledHealthCheckPreferenceStatus) -> bool {
        match self.status {
            ScheduledHealthCheckPreferenceStatus::New => {
                return next == ScheduledHealthCheckPreferenceStatus::Done;
            }
            ScheduledHealthCheckPreferenceStatus::Queued => {
                return next == ScheduledHealthCheckPreferenceStatus::Done;
            }
            ScheduledHealthCheckPreferenceStatus::Running => {
                match next {
                    ScheduledHealthCheckPreferenceStatus::Done | ScheduledHealthCheckPreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledHealthCheckPreferenceStatus::Done => {
                match next {
                    ScheduledHealthCheckPreferenceStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledHealthCheckPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled health check preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled health check preference", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.starts_at as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledHealthCheckPreference {
        ScheduledHealthCheckPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            scope: self.scope.clone(),
            fingerprint: self.fingerprint.clone(),
            starts_at: self.starts_at,
            external_ref: self.external_ref.clone(),
            region: self.region.clone(),
            namespace: self.namespace.clone(),
            weight: self.weight,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a scheduled health check preference in status new.
pub fn new_scheduled_health_check_preference(code: &str, title: &str) -> ScheduledHealthCheckPreference {
    ScheduledHealthCheckPreference {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        scope: "".to_string(),
        fingerprint: "".to_string(),
        starts_at: 0,
        external_ref: "".to_string(),
        region: "".to_string(),
        namespace: "".to_string(),
        weight: 0.0,
        revision: 0,
        status: ScheduledHealthCheckPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_health_check_preference("code-202", "title-778");
        assert_eq!(item.status, ScheduledHealthCheckPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_scheduled_health_check_preference("code-202", "title-778");
        if item.move_to(ScheduledHealthCheckPreferenceStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledHealthCheckPreferenceStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_health_check_preference("code-202", "title-778");
        item.status = ScheduledHealthCheckPreferenceStatus::Failed;
        assert!(
            item.move_to(ScheduledHealthCheckPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
