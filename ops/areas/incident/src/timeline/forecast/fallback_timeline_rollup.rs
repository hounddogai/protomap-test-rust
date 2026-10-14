use super::*;

use acme_kit::text;

/// Where a fallback timeline rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTimelineRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackTimelineRollupStatus {
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

impl std::fmt::Display for FallbackTimelineRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback timeline rollup as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTimelineRollup {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub maximum: f64,
    pub namespace: String,
    pub expires_at: i64,
    pub status: FallbackTimelineRollupStatus,
}

impl FallbackTimelineRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTimelineRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTimelineRollupStatus) -> bool {
        match self.status {
            FallbackTimelineRollupStatus::Draft => {
                return next == FallbackTimelineRollupStatus::Closed;
            }
            FallbackTimelineRollupStatus::Pending => {
                return next == FallbackTimelineRollupStatus::Active || next == FallbackTimelineRollupStatus::Suspended;
            }
            FallbackTimelineRollupStatus::Active => {
                match next {
                    FallbackTimelineRollupStatus::Suspended | FallbackTimelineRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTimelineRollupStatus::Suspended => {
                return next == FallbackTimelineRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackTimelineRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback timeline rollup {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback timeline rollup", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.expires_at as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackTimelineRollup {
        FallbackTimelineRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            maximum: self.maximum,
            namespace: self.namespace.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback timeline rollup in status draft.
pub fn new_fallback_timeline_rollup(code: &str, label: &str) -> FallbackTimelineRollup {
    FallbackTimelineRollup {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        maximum: 0.0,
        namespace: "".to_string(),
        expires_at: 0,
        status: FallbackTimelineRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_timeline_rollup("code-623", "label-803");
        assert_eq!(item.status, FallbackTimelineRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_fallback_timeline_rollup("code-623", "label-803");
        if item.move_to(FallbackTimelineRollupStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTimelineRollupStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_timeline_rollup("code-623", "label-803");
        item.status = FallbackTimelineRollupStatus::Closed;
        assert!(
            item.move_to(FallbackTimelineRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
