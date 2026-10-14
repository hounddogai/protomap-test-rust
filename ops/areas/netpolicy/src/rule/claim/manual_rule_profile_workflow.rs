use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves manual rule profiles through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct ManualRuleProfileDesk {
    pub store: Rc<dyn ManualRuleProfileStore>,
    pub validator: ManualRuleProfileValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl ManualRuleProfileDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn start(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual rule profile {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(ManualRuleProfileStatus::Approved)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is approved", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("manual rule profile {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(ManualRuleProfileStatus::Withdrawn)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is withdrawn: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_manual_rule_profile_desk(
    store: Rc<dyn ManualRuleProfileStore>,
    clock: Rc<dyn clock::Clock>,
) -> ManualRuleProfileDesk {
    ManualRuleProfileDesk {
        store: store.clone(),
        validator: default_manual_rule_profile_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
