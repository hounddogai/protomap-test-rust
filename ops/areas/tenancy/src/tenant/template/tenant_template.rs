use super::*;

use acme_kit::text;

/// Where a tenant template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TenantTemplateStatus {
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

impl std::fmt::Display for TenantTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant template as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplate {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub cluster: String,
    pub score: f64,
    pub ratio: f64,
    pub status: TenantTemplateStatus,
}

impl TenantTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantTemplateStatus) -> bool {
        match self.status {
            TenantTemplateStatus::New => {
                return next == TenantTemplateStatus::Done;
            }
            TenantTemplateStatus::Queued => {
                return next == TenantTemplateStatus::Running || next == TenantTemplateStatus::Done;
            }
            TenantTemplateStatus::Running => {
                match next {
                    TenantTemplateStatus::Done | TenantTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantTemplateStatus::Done => {
                match next {
                    TenantTemplateStatus::Failed => {
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

    pub fn move_to(&mut self, next: TenantTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tenant template", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ratio + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantTemplate {
        TenantTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            cluster: self.cluster.clone(),
            score: self.score,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tenant template in status new.
pub fn new_tenant_template(code: &str, label: &str) -> TenantTemplate {
    TenantTemplate {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        cluster: "".to_string(),
        score: 0.0,
        ratio: 0.0,
        status: TenantTemplateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tenant_template("code-516", "label-117");
        assert_eq!(item.status, TenantTemplateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_tenant_template("code-516", "label-117");
        if item.move_to(TenantTemplateStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantTemplateStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tenant_template("code-516", "label-117");
        item.status = TenantTemplateStatus::Failed;
        assert!(
            item.move_to(TenantTemplateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
