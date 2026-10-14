use super::*;

use acme_kit::text;

/// Where a quote milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuoteMilestoneStatus {
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

impl std::fmt::Display for QuoteMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMilestone {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub expires_at: i64,
    pub cost: f64,
    pub burst: i64,
    pub namespace: String,
    pub batch_size: i64,
    pub host: String,
    pub audience: String,
    pub status: QuoteMilestoneStatus,
}

impl QuoteMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteMilestoneStatus) -> bool {
        match self.status {
            QuoteMilestoneStatus::Idle => {
                match next {
                    QuoteMilestoneStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteMilestoneStatus::Warming => {
                return next == QuoteMilestoneStatus::Ready || next == QuoteMilestoneStatus::Stopped;
            }
            QuoteMilestoneStatus::Ready => {
                match next {
                    QuoteMilestoneStatus::Draining | QuoteMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteMilestoneStatus::Draining => {
                return next == QuoteMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.batch_size as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteMilestone {
        QuoteMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            expires_at: self.expires_at,
            cost: self.cost,
            burst: self.burst,
            namespace: self.namespace.clone(),
            batch_size: self.batch_size,
            host: self.host.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a quote milestone in status idle.
pub fn new_quote_milestone(code: &str, label: &str) -> QuoteMilestone {
    QuoteMilestone {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        expires_at: 0,
        cost: 0.0,
        burst: 0,
        namespace: "".to_string(),
        batch_size: 0,
        host: "".to_string(),
        audience: "".to_string(),
        status: QuoteMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_quote_milestone("code-921", "label-178");
        assert_eq!(item.status, QuoteMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_quote_milestone("code-921", "label-178");
        if item.move_to(QuoteMilestoneStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteMilestoneStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_quote_milestone("code-921", "label-178");
        item.status = QuoteMilestoneStatus::Stopped;
        assert!(
            item.move_to(QuoteMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
