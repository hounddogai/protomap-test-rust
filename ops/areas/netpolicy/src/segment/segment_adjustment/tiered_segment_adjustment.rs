use super::*;

use acme_kit::text;

/// Where a tiered segment adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSegmentAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredSegmentAdjustmentStatus {
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

impl std::fmt::Display for TieredSegmentAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered segment adjustment as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAdjustment {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub weight: f64,
    pub port: i64,
    pub discount: f64,
    pub locale: String,
    pub ceiling: f64,
    pub starts_at: i64,
    pub score: f64,
    pub status: TieredSegmentAdjustmentStatus,
}

impl TieredSegmentAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSegmentAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSegmentAdjustmentStatus) -> bool {
        match self.status {
            TieredSegmentAdjustmentStatus::Idle => {
                return next == TieredSegmentAdjustmentStatus::Ready;
            }
            TieredSegmentAdjustmentStatus::Warming => {
                return next == TieredSegmentAdjustmentStatus::Ready;
            }
            TieredSegmentAdjustmentStatus::Ready => {
                match next {
                    TieredSegmentAdjustmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSegmentAdjustmentStatus::Draining => {
                match next {
                    TieredSegmentAdjustmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredSegmentAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered segment adjustment {} cannot move from {} to {}",
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
        format!("{} {}", "tiered segment adjustment", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.discount * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSegmentAdjustment {
        TieredSegmentAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            weight: self.weight,
            port: self.port,
            discount: self.discount,
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            starts_at: self.starts_at,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered segment adjustment in status idle.
pub fn new_tiered_segment_adjustment(code: &str, title: &str) -> TieredSegmentAdjustment {
    TieredSegmentAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        weight: 0.0,
        port: 0,
        discount: 0.0,
        locale: "".to_string(),
        ceiling: 0.0,
        starts_at: 0,
        score: 0.0,
        status: TieredSegmentAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_segment_adjustment("code-896", "title-295");
        assert_eq!(item.status, TieredSegmentAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_tiered_segment_adjustment("code-896", "title-295");
        if item.move_to(TieredSegmentAdjustmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredSegmentAdjustmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_segment_adjustment("code-896", "title-295");
        item.status = TieredSegmentAdjustmentStatus::Stopped;
        assert!(
            item.move_to(TieredSegmentAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
