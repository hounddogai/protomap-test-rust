use super::*;

use acme_kit::text;

/// Where a fallback segment window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSegmentWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackSegmentWindowStatus {
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

impl std::fmt::Display for FallbackSegmentWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback segment window as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentWindow {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub fingerprint: String,
    pub category: String,
    pub ends_at: i64,
    pub pinned: bool,
    pub host: String,
    pub threshold: f64,
    pub key_id: String,
    pub status: FallbackSegmentWindowStatus,
}

impl FallbackSegmentWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSegmentWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSegmentWindowStatus) -> bool {
        match self.status {
            FallbackSegmentWindowStatus::Idle => {
                match next {
                    FallbackSegmentWindowStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentWindowStatus::Warming => {
                match next {
                    FallbackSegmentWindowStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentWindowStatus::Ready => {
                match next {
                    FallbackSegmentWindowStatus::Draining | FallbackSegmentWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentWindowStatus::Draining => {
                match next {
                    FallbackSegmentWindowStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackSegmentWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback segment window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ends_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSegmentWindow {
        FallbackSegmentWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            ends_at: self.ends_at,
            pinned: self.pinned,
            host: self.host.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback segment window in status idle.
pub fn new_fallback_segment_window(code: &str, title: &str) -> FallbackSegmentWindow {
    FallbackSegmentWindow {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        fingerprint: "".to_string(),
        category: "".to_string(),
        ends_at: 0,
        pinned: false,
        host: "".to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        status: FallbackSegmentWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_segment_window("code-506", "title-69");
        assert_eq!(item.status, FallbackSegmentWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_fallback_segment_window("code-506", "title-69");
        if item.move_to(FallbackSegmentWindowStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSegmentWindowStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_segment_window("code-506", "title-69");
        item.status = FallbackSegmentWindowStatus::Stopped;
        assert!(
            item.move_to(FallbackSegmentWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
