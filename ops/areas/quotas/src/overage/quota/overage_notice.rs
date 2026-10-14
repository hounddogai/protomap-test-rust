use super::*;

use acme_kit::text;

/// Where a overage notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl OverageNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for OverageNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage notice as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageNotice {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub description: String,
    pub region: String,
    pub quantity: i64,
    pub approved: bool,
    pub checksum: String,
    pub segment: String,
    pub window_days: i64,
    pub status: OverageNoticeStatus,
}

impl OverageNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageNoticeStatus) -> bool {
        match self.status {
            OverageNoticeStatus::Created => {
                return next == OverageNoticeStatus::Validated || next == OverageNoticeStatus::Published;
            }
            OverageNoticeStatus::Validated => {
                match next {
                    OverageNoticeStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageNoticeStatus::Published => {
                return next == OverageNoticeStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageNotice {
        OverageNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            description: self.description.clone(),
            region: self.region.clone(),
            quantity: self.quantity,
            approved: self.approved,
            checksum: self.checksum.clone(),
            segment: self.segment.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a overage notice in status created.
pub fn new_overage_notice(code: &str, title: &str) -> OverageNotice {
    OverageNotice {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        description: "".to_string(),
        region: "".to_string(),
        quantity: 0,
        approved: false,
        checksum: "".to_string(),
        segment: "".to_string(),
        window_days: 0,
        status: OverageNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_overage_notice("code-130", "title-707");
        assert_eq!(item.status, OverageNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_overage_notice("code-130", "title-707");
        if item.move_to(OverageNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_overage_notice("code-130", "title-707");
        item.status = OverageNoticeStatus::Retired;
        assert!(
            item.move_to(OverageNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
