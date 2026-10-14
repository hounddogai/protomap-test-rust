use super::*;

use acme_kit::text;

/// Where a limit segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitSegmentStatus {
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

impl std::fmt::Display for LimitSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit segment as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitSegment {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub floor_value: f64,
    pub cipher: String,
    pub checksum: String,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub description: String,
    pub approved: bool,
    pub status: LimitSegmentStatus,
}

impl LimitSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitSegmentStatus) -> bool {
        match self.status {
            LimitSegmentStatus::Idle => {
                return next == LimitSegmentStatus::Ready;
            }
            LimitSegmentStatus::Warming => {
                return next == LimitSegmentStatus::Stopped;
            }
            LimitSegmentStatus::Ready => {
                return next == LimitSegmentStatus::Stopped;
            }
            LimitSegmentStatus::Draining => {
                return next == LimitSegmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.batch_size as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitSegment {
        LimitSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            cipher: self.cipher.clone(),
            checksum: self.checksum.clone(),
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            description: self.description.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a limit segment in status idle.
pub fn new_limit_segment(code: &str, title: &str) -> LimitSegment {
    LimitSegment {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        floor_value: 0.0,
        cipher: "".to_string(),
        checksum: "".to_string(),
        batch_size: 0,
        tags: Vec::new(),
        description: "".to_string(),
        approved: false,
        status: LimitSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_limit_segment("code-614", "title-558");
        assert_eq!(item.status, LimitSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_limit_segment("code-614", "title-558");
        if item.move_to(LimitSegmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitSegmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_limit_segment("code-614", "title-558");
        item.status = LimitSegmentStatus::Stopped;
        assert!(item.move_to(LimitSegmentStatus::Idle).is_err(), "expected move to fail");
    }
}
