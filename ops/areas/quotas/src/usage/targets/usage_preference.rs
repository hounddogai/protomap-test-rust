use super::*;

use acme_kit::text;

/// Where a usage preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsagePreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl UsagePreferenceStatus {
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

impl std::fmt::Display for UsagePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage preference as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsagePreference {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub ttl_seconds: i64,
    pub cluster: String,
    pub visible: bool,
    pub minimum: f64,
    pub limit: i64,
    pub status: UsagePreferenceStatus,
}

impl UsagePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsagePreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsagePreferenceStatus) -> bool {
        match self.status {
            UsagePreferenceStatus::Idle => {
                match next {
                    UsagePreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsagePreferenceStatus::Warming => {
                match next {
                    UsagePreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsagePreferenceStatus::Ready => {
                return next == UsagePreferenceStatus::Draining;
            }
            UsagePreferenceStatus::Draining => {
                return next == UsagePreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsagePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.batch_size as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsagePreference {
        UsagePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            ttl_seconds: self.ttl_seconds,
            cluster: self.cluster.clone(),
            visible: self.visible,
            minimum: self.minimum,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a usage preference in status idle.
pub fn new_usage_preference(code: &str, title: &str) -> UsagePreference {
    UsagePreference {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        ttl_seconds: 0,
        cluster: "".to_string(),
        visible: false,
        minimum: 0.0,
        limit: 0,
        status: UsagePreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_usage_preference("code-785", "title-95");
        assert_eq!(item.status, UsagePreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_usage_preference("code-785", "title-95");
        if item.move_to(UsagePreferenceStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsagePreferenceStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_usage_preference("code-785", "title-95");
        item.status = UsagePreferenceStatus::Stopped;
        assert!(
            item.move_to(UsagePreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
