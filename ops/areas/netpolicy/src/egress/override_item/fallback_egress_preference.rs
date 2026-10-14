use super::*;

use acme_kit::text;

/// Where a fallback egress preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEgressPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackEgressPreferenceStatus {
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

impl std::fmt::Display for FallbackEgressPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback egress preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPreference {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub timezone: String,
    pub summary: String,
    pub cluster: String,
    pub starts_at: i64,
    pub grace_days: i64,
    pub window_days: i64,
    pub unit: String,
    pub ratio: f64,
    pub status: FallbackEgressPreferenceStatus,
}

impl FallbackEgressPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEgressPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEgressPreferenceStatus) -> bool {
        match self.status {
            FallbackEgressPreferenceStatus::New => {
                match next {
                    FallbackEgressPreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressPreferenceStatus::Queued => {
                match next {
                    FallbackEgressPreferenceStatus::Running | FallbackEgressPreferenceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressPreferenceStatus::Running => {
                return next == FallbackEgressPreferenceStatus::Done;
            }
            FallbackEgressPreferenceStatus::Done => {
                match next {
                    FallbackEgressPreferenceStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackEgressPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback egress preference {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.starts_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackEgressPreference {
        FallbackEgressPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            timezone: self.timezone.clone(),
            summary: self.summary.clone(),
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            grace_days: self.grace_days,
            window_days: self.window_days,
            unit: self.unit.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback egress preference in status new.
pub fn new_fallback_egress_preference(code: &str, title: &str) -> FallbackEgressPreference {
    FallbackEgressPreference {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        timezone: "".to_string(),
        summary: "".to_string(),
        cluster: "".to_string(),
        starts_at: 0,
        grace_days: 0,
        window_days: 0,
        unit: "".to_string(),
        ratio: 0.0,
        status: FallbackEgressPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_egress_preference("code-613", "title-490");
        assert_eq!(item.status, FallbackEgressPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_fallback_egress_preference("code-613", "title-490");
        if item.move_to(FallbackEgressPreferenceStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackEgressPreferenceStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_egress_preference("code-613", "title-490");
        item.status = FallbackEgressPreferenceStatus::Failed;
        assert!(
            item.move_to(FallbackEgressPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
