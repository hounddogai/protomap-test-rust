use super::*;

use acme_kit::text;

/// Where a primary ttl policy plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTtlPolicyPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryTtlPolicyPlanStatus {
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

impl std::fmt::Display for PrimaryTtlPolicyPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary ttl policy plan as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyPlan {
    pub code: String,
    pub title: String,
    pub source: String,
    pub summary: String,
    pub fingerprint: String,
    pub tags: Vec<String>,
    pub contact_email: String,
    pub approved: bool,
    pub score: f64,
    pub maximum: f64,
    pub label: String,
    pub status: PrimaryTtlPolicyPlanStatus,
}

impl PrimaryTtlPolicyPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTtlPolicyPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTtlPolicyPlanStatus) -> bool {
        match self.status {
            PrimaryTtlPolicyPlanStatus::New => {
                match next {
                    PrimaryTtlPolicyPlanStatus::Queued | PrimaryTtlPolicyPlanStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyPlanStatus::Queued => {
                return next == PrimaryTtlPolicyPlanStatus::Done || next == PrimaryTtlPolicyPlanStatus::Failed;
            }
            PrimaryTtlPolicyPlanStatus::Running => {
                match next {
                    PrimaryTtlPolicyPlanStatus::Done | PrimaryTtlPolicyPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyPlanStatus::Done => {
                return next == PrimaryTtlPolicyPlanStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTtlPolicyPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary ttl policy plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary ttl policy plan", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.score + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTtlPolicyPlan {
        PrimaryTtlPolicyPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            summary: self.summary.clone(),
            fingerprint: self.fingerprint.clone(),
            tags: self.tags.clone(),
            contact_email: self.contact_email.clone(),
            approved: self.approved,
            score: self.score,
            maximum: self.maximum,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary ttl policy plan in status new.
pub fn new_primary_ttl_policy_plan(code: &str, title: &str) -> PrimaryTtlPolicyPlan {
    PrimaryTtlPolicyPlan {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        summary: "".to_string(),
        fingerprint: "".to_string(),
        tags: Vec::new(),
        contact_email: "".to_string(),
        approved: false,
        score: 0.0,
        maximum: 0.0,
        label: "".to_string(),
        status: PrimaryTtlPolicyPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_ttl_policy_plan("code-493", "title-974");
        assert_eq!(item.status, PrimaryTtlPolicyPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_primary_ttl_policy_plan("code-493", "title-974");
        if item.move_to(PrimaryTtlPolicyPlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTtlPolicyPlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_ttl_policy_plan("code-493", "title-974");
        item.status = PrimaryTtlPolicyPlanStatus::Failed;
        assert!(
            item.move_to(PrimaryTtlPolicyPlanStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
