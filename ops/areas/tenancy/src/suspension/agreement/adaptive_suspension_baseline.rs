use super::*;

use acme_kit::text;

/// Where a adaptive suspension baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSuspensionBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveSuspensionBaselineStatus {
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

impl std::fmt::Display for AdaptiveSuspensionBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive suspension baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionBaseline {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub key_id: String,
    pub summary: String,
    pub margin: f64,
    pub approved: bool,
    pub rate: f64,
    pub pinned: bool,
    pub status: AdaptiveSuspensionBaselineStatus,
}

impl AdaptiveSuspensionBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSuspensionBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSuspensionBaselineStatus) -> bool {
        match self.status {
            AdaptiveSuspensionBaselineStatus::Idle => {
                match next {
                    AdaptiveSuspensionBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSuspensionBaselineStatus::Warming => {
                return next == AdaptiveSuspensionBaselineStatus::Ready
                    || next == AdaptiveSuspensionBaselineStatus::Draining;
            }
            AdaptiveSuspensionBaselineStatus::Ready => {
                match next {
                    AdaptiveSuspensionBaselineStatus::Draining | AdaptiveSuspensionBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSuspensionBaselineStatus::Draining => {
                match next {
                    AdaptiveSuspensionBaselineStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AdaptiveSuspensionBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive suspension baseline {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.rate * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSuspensionBaseline {
        AdaptiveSuspensionBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            summary: self.summary.clone(),
            margin: self.margin,
            approved: self.approved,
            rate: self.rate,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive suspension baseline in status idle.
pub fn new_adaptive_suspension_baseline(code: &str, name: &str) -> AdaptiveSuspensionBaseline {
    AdaptiveSuspensionBaseline {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        key_id: "".to_string(),
        summary: "".to_string(),
        margin: 0.0,
        approved: false,
        rate: 0.0,
        pinned: false,
        status: AdaptiveSuspensionBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_suspension_baseline("code-355", "name-292");
        assert_eq!(item.status, AdaptiveSuspensionBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_adaptive_suspension_baseline("code-355", "name-292");
        if item.move_to(AdaptiveSuspensionBaselineStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSuspensionBaselineStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_suspension_baseline("code-355", "name-292");
        item.status = AdaptiveSuspensionBaselineStatus::Stopped;
        assert!(
            item.move_to(AdaptiveSuspensionBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
