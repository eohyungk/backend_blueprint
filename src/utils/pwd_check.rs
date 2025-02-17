use crate::error::{Error, Result};

// region: --- Password Validation Types
#[derive(Debug)]
pub struct PwdValidation {
    pub has_min_len: bool,
    pub has_uppercase: bool,
    pub has_lowercase: bool,
    pub has_number: bool,
    pub has_special_char: bool,
}

impl PwdValidation {
    pub fn new(pwd: &str) -> Self {
        Self {
            has_min_len: pwd.len() >= 8,
            has_uppercase: pwd.chars().any(|c| c.is_uppercase()),
            has_lowercase: pwd.chars().any(|c| c.is_lowercase()),
            has_number: pwd.chars().any(|c| c.is_numeric()),
            has_special_char: pwd.chars().any(|c| !c.is_alphanumeric()),
        }
    }

    pub fn validate(&self) -> Result<()> {
        let mut issues = Vec::new();

        if !self.has_min_len {
            issues.push("be at least 8 characters");
        }
        if !self.has_uppercase {
            issues.push("include at least one uppercase letter");
        }
        if !self.has_lowercase {
            issues.push("include at least one lowercase letter");
        }
        if !self.has_number {
            issues.push("include at least one number");
        }
        if !self.has_special_char {
            issues.push("include at least one special character");
        }

        if !issues.is_empty() {
            return Err(Error::PwdValidationFailed { 
                issues: issues.join(", ")
            });
        }

        Ok(())
    }
}
// endregion: --- Password Validation Types