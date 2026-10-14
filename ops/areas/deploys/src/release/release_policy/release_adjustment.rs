use super::*;

use acme_kit::text;

/// Where a release adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReleaseAdjustmentStatus {
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

impl std::fmt::Display for ReleaseAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release adjustment as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustment {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub reviewer: String,
    pub country: String,
    pub owner_id: String,
    pub source: String,
    pub updated_at: i64,
    pub status: ReleaseAdjustmentStatus,
}

impl ReleaseAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseAdjustmentStatus) -> bool {
        match self.status {
            ReleaseAdjustmentStatus::Idle => {
                return next == ReleaseAdjustmentStatus::Draining || next == ReleaseAdjustmentStatus::Stopped;
            }
            ReleaseAdjustmentStatus::Warming => {
                return next == ReleaseAdjustmentStatus::Ready;
            }
            ReleaseAdjustmentStatus::Ready => {
                return next == ReleaseAdjustmentStatus::Draining || next == ReleaseAdjustmentStatus::Stopped;
            }
            ReleaseAdjustmentStatus::Draining => {
                return next == ReleaseAdjustmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.updated_at as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseAdjustment {
        ReleaseAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            reviewer: self.reviewer.clone(),
            country: self.country.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a release adjustment in status idle.
pub fn new_release_adjustment(code: &str, title: &str) -> ReleaseAdjustment {
    ReleaseAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        reviewer: "".to_string(),
        country: "".to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        updated_at: 0,
        status: ReleaseAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_release_adjustment("code-601", "title-667");
        assert_eq!(item.status, ReleaseAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_release_adjustment("code-601", "title-667");
        if item.move_to(ReleaseAdjustmentStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseAdjustmentStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_release_adjustment("code-601", "title-667");
        item.status = ReleaseAdjustmentStatus::Stopped;
        assert!(
            item.move_to(ReleaseAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
