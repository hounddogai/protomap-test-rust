use super::*;

use acme_kit::text;

/// Where a regional freeze tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFreezeTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalFreezeTierStatus {
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

impl std::fmt::Display for RegionalFreezeTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional freeze tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeTier {
    pub code: String,
    pub label: String,
    pub host: String,
    pub segment: String,
    pub cipher: String,
    pub rotation_days: i64,
    pub notes: String,
    pub capacity: i64,
    pub expires_at: i64,
    pub cost: f64,
    pub status: RegionalFreezeTierStatus,
}

impl RegionalFreezeTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFreezeTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFreezeTierStatus) -> bool {
        match self.status {
            RegionalFreezeTierStatus::Open => {
                return next == RegionalFreezeTierStatus::InReview;
            }
            RegionalFreezeTierStatus::InReview => {
                match next {
                    RegionalFreezeTierStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFreezeTierStatus::Resolved => {
                match next {
                    RegionalFreezeTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFreezeTierStatus::Reopened => {
                match next {
                    RegionalFreezeTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalFreezeTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional freeze tier {} cannot move from {} to {}",
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
        format!("{} {}", "regional freeze tier", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.capacity as f64 - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalFreezeTier {
        RegionalFreezeTier {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            segment: self.segment.clone(),
            cipher: self.cipher.clone(),
            rotation_days: self.rotation_days,
            notes: self.notes.clone(),
            capacity: self.capacity,
            expires_at: self.expires_at,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional freeze tier in status open.
pub fn new_regional_freeze_tier(code: &str, label: &str) -> RegionalFreezeTier {
    RegionalFreezeTier {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        segment: "".to_string(),
        cipher: "".to_string(),
        rotation_days: 0,
        notes: "".to_string(),
        capacity: 0,
        expires_at: 0,
        cost: 0.0,
        status: RegionalFreezeTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_freeze_tier("code-496", "label-321");
        assert_eq!(item.status, RegionalFreezeTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_regional_freeze_tier("code-496", "label-321");
        if item.move_to(RegionalFreezeTierStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFreezeTierStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_freeze_tier("code-496", "label-321");
        item.status = RegionalFreezeTierStatus::Archived;
        assert!(
            item.move_to(RegionalFreezeTierStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
