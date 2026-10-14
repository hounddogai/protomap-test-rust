use super::*;

use acme_kit::text;

/// Where a region pin target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionPinTargetStatus {
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

impl std::fmt::Display for RegionPinTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinTarget {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub source: String,
    pub ratio: f64,
    pub rate: f64,
    pub approved: bool,
    pub margin: f64,
    pub capacity: i64,
    pub status: RegionPinTargetStatus,
}

impl RegionPinTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinTargetStatus) -> bool {
        match self.status {
            RegionPinTargetStatus::New => {
                return next == RegionPinTargetStatus::Queued || next == RegionPinTargetStatus::Done;
            }
            RegionPinTargetStatus::Queued => {
                match next {
                    RegionPinTargetStatus::Done | RegionPinTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinTargetStatus::Running => {
                return next == RegionPinTargetStatus::Done || next == RegionPinTargetStatus::Failed;
            }
            RegionPinTargetStatus::Done => {
                match next {
                    RegionPinTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionPinTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin target", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.capacity as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinTarget {
        RegionPinTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            source: self.source.clone(),
            ratio: self.ratio,
            rate: self.rate,
            approved: self.approved,
            margin: self.margin,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a region pin target in status new.
pub fn new_region_pin_target(code: &str, title: &str) -> RegionPinTarget {
    RegionPinTarget {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        source: "".to_string(),
        ratio: 0.0,
        rate: 0.0,
        approved: false,
        margin: 0.0,
        capacity: 0,
        status: RegionPinTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_region_pin_target("code-680", "title-629");
        assert_eq!(item.status, RegionPinTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_region_pin_target("code-680", "title-629");
        if item.move_to(RegionPinTargetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinTargetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_region_pin_target("code-680", "title-629");
        item.status = RegionPinTargetStatus::Failed;
        assert!(
            item.move_to(RegionPinTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
