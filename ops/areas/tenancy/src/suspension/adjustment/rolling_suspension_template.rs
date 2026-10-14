use super::*;

use acme_kit::text;

/// Where a rolling suspension template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSuspensionTemplateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingSuspensionTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingSuspensionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling suspension template as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionTemplate {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub cipher: String,
    pub budget: f64,
    pub window_days: i64,
    pub status: RollingSuspensionTemplateStatus,
}

impl RollingSuspensionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSuspensionTemplateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSuspensionTemplateStatus) -> bool {
        match self.status {
            RollingSuspensionTemplateStatus::Proposed => {
                return next == RollingSuspensionTemplateStatus::Approved;
            }
            RollingSuspensionTemplateStatus::Approved => {
                return next == RollingSuspensionTemplateStatus::Rejected
                    || next == RollingSuspensionTemplateStatus::Withdrawn;
            }
            RollingSuspensionTemplateStatus::Rejected => {
                match next {
                    RollingSuspensionTemplateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingSuspensionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling suspension template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling suspension template", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        (self.budget * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSuspensionTemplate {
        RollingSuspensionTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            cipher: self.cipher.clone(),
            budget: self.budget,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling suspension template in status proposed.
pub fn new_rolling_suspension_template(code: &str, title: &str) -> RollingSuspensionTemplate {
    RollingSuspensionTemplate {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        cipher: "".to_string(),
        budget: 0.0,
        window_days: 0,
        status: RollingSuspensionTemplateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_suspension_template("code-729", "title-452");
        assert_eq!(item.status, RollingSuspensionTemplateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_suspension_template("code-729", "title-452");
        if item.move_to(RollingSuspensionTemplateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSuspensionTemplateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_suspension_template("code-729", "title-452");
        item.status = RollingSuspensionTemplateStatus::Withdrawn;
        assert!(
            item.move_to(RollingSuspensionTemplateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
