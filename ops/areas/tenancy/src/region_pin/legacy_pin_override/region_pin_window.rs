use super::*;

use acme_kit::text;

/// Where a region pin window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionPinWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionPinWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin window as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinWindow {
    pub code: String,
    pub title: String,
    pub source: String,
    pub namespace: String,
    pub category: String,
    pub owner_id: String,
    pub weight: f64,
    pub maximum: f64,
    pub status: RegionPinWindowStatus,
}

impl RegionPinWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinWindowStatus) -> bool {
        match self.status {
            RegionPinWindowStatus::Open => {
                match next {
                    RegionPinWindowStatus::Resolved | RegionPinWindowStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinWindowStatus::InReview => {
                match next {
                    RegionPinWindowStatus::Resolved | RegionPinWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinWindowStatus::Resolved => {
                match next {
                    RegionPinWindowStatus::Reopened | RegionPinWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinWindowStatus::Reopened => {
                return next == RegionPinWindowStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin window", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.maximum + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinWindow {
        RegionPinWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            namespace: self.namespace.clone(),
            category: self.category.clone(),
            owner_id: self.owner_id.clone(),
            weight: self.weight,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a region pin window in status open.
pub fn new_region_pin_window(code: &str, title: &str) -> RegionPinWindow {
    RegionPinWindow {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        namespace: "".to_string(),
        category: "".to_string(),
        owner_id: "".to_string(),
        weight: 0.0,
        maximum: 0.0,
        status: RegionPinWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_region_pin_window("code-508", "title-438");
        assert_eq!(item.status, RegionPinWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_region_pin_window("code-508", "title-438");
        if item.move_to(RegionPinWindowStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinWindowStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_region_pin_window("code-508", "title-438");
        item.status = RegionPinWindowStatus::Archived;
        assert!(
            item.move_to(RegionPinWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
