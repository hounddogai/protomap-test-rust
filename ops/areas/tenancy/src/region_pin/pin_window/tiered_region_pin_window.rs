use super::*;

use acme_kit::text;

/// Where a tiered region pin window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRegionPinWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredRegionPinWindowStatus {
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

impl std::fmt::Display for TieredRegionPinWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered region pin window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinWindow {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub ttl_seconds: i64,
    pub floor_value: f64,
    pub status: TieredRegionPinWindowStatus,
}

impl TieredRegionPinWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRegionPinWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRegionPinWindowStatus) -> bool {
        match self.status {
            TieredRegionPinWindowStatus::Draft => {
                match next {
                    TieredRegionPinWindowStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRegionPinWindowStatus::Pending => {
                match next {
                    TieredRegionPinWindowStatus::Active | TieredRegionPinWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRegionPinWindowStatus::Active => {
                match next {
                    TieredRegionPinWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRegionPinWindowStatus::Suspended => {
                match next {
                    TieredRegionPinWindowStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredRegionPinWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered region pin window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered region pin window", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRegionPinWindow {
        TieredRegionPinWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            ttl_seconds: self.ttl_seconds,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered region pin window in status draft.
pub fn new_tiered_region_pin_window(code: &str, title: &str) -> TieredRegionPinWindow {
    TieredRegionPinWindow {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        ttl_seconds: 0,
        floor_value: 0.0,
        status: TieredRegionPinWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_region_pin_window("code-787", "title-673");
        assert_eq!(item.status, TieredRegionPinWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_tiered_region_pin_window("code-787", "title-673");
        if item.move_to(TieredRegionPinWindowStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRegionPinWindowStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_region_pin_window("code-787", "title-673");
        item.status = TieredRegionPinWindowStatus::Closed;
        assert!(
            item.move_to(TieredRegionPinWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
