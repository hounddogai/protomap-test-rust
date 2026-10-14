use super::*;

use acme_kit::text;

/// Where a lenient plan window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPlanWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientPlanWindowStatus {
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

impl std::fmt::Display for LenientPlanWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient plan window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanWindow {
    pub code: String,
    pub label: String,
    pub name: String,
    pub reviewer: String,
    pub audience: String,
    pub discount: f64,
    pub retries: i64,
    pub cipher: String,
    pub batch_size: i64,
    pub score: f64,
    pub duration_minutes: i64,
    pub status: LenientPlanWindowStatus,
}

impl LenientPlanWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPlanWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPlanWindowStatus) -> bool {
        match self.status {
            LenientPlanWindowStatus::Draft => {
                match next {
                    LenientPlanWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanWindowStatus::Pending => {
                match next {
                    LenientPlanWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanWindowStatus::Active => {
                match next {
                    LenientPlanWindowStatus::Suspended | LenientPlanWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanWindowStatus::Suspended => {
                return next == LenientPlanWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPlanWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient plan window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.duration_minutes as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPlanWindow {
        LenientPlanWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            discount: self.discount,
            retries: self.retries,
            cipher: self.cipher.clone(),
            batch_size: self.batch_size,
            score: self.score,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient plan window in status draft.
pub fn new_lenient_plan_window(code: &str, label: &str) -> LenientPlanWindow {
    LenientPlanWindow {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        reviewer: "".to_string(),
        audience: "".to_string(),
        discount: 0.0,
        retries: 0,
        cipher: "".to_string(),
        batch_size: 0,
        score: 0.0,
        duration_minutes: 0,
        status: LenientPlanWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_plan_window("code-373", "label-467");
        assert_eq!(item.status, LenientPlanWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_lenient_plan_window("code-373", "label-467");
        if item.move_to(LenientPlanWindowStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPlanWindowStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_plan_window("code-373", "label-467");
        item.status = LenientPlanWindowStatus::Closed;
        assert!(
            item.move_to(LenientPlanWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
