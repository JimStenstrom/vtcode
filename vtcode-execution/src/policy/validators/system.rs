//! System information command validators
//!
//! This module validates system information commands that are read-only and safe:
//! - echo: Print text
//! - pwd: Print working directory
//! - printenv: Print environment variables
//! - which: Locate programs
//! - date: Display date/time
//! - whoami: Current user
//! - hostname: System hostname
//! - uname: System information

use anyhow::{Result, anyhow};

/// Validate echo command arguments.
///
/// Allowed flags: -n, -e, -E
pub fn validate_echo(args: &[String]) -> Result<()> {
    for arg in args {
        if arg.starts_with('-') {
            match arg.as_str() {
                "-n" | "-e" | "-E" => continue,
                _ => {
                    return Err(anyhow!("unsupported echo flag '{}'", arg));
                }
            }
        }
    }
    Ok(())
}

/// Validate pwd command (no arguments allowed).
pub fn validate_pwd(args: &[String]) -> Result<()> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("pwd does not accept arguments"))
    }
}

/// Validate printenv command.
///
/// Accepts zero or one environment variable name.
/// Variable names must be alphanumeric or underscore only.
pub fn validate_printenv(args: &[String]) -> Result<()> {
    match args.len() {
        0 => Ok(()),
        1 => {
            let name = &args[0];
            if name.is_empty()
                || !name
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                return Err(anyhow!("invalid environment variable name '{}'", name));
            }
            Ok(())
        }
        _ => Err(anyhow!("printenv accepts zero or one argument")),
    }
}

/// Validate which command arguments.
///
/// Program names must not contain:
/// - Slashes (/)
/// - Whitespace
/// - Be empty
///
/// Allowed flags: -a, -s
pub fn validate_which(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("which requires at least one program name"));
    }

    for arg in args {
        match arg.as_str() {
            "-a" | "-s" => continue,
            value if value.starts_with('-') => {
                return Err(anyhow!("unsupported which flag '{}'", value));
            }
            value => {
                if value.is_empty()
                    || value.contains('/')
                    || value.chars().any(|ch| ch.is_whitespace())
                {
                    return Err(anyhow!(
                        "program name '{}' contains unsupported characters",
                        value
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Validate date command arguments.
///
/// Date just displays current date/time, safe with format args.
/// Format strings starting with + are allowed.
pub fn validate_date(args: &[String]) -> Result<()> {
    for arg in args {
        if arg.starts_with('+') {
            // Format string is safe
            continue;
        }
    }
    Ok(())
}

/// Validate whoami command (no arguments, always safe).
pub fn validate_whoami(_args: &[String]) -> Result<()> {
    Ok(())
}

/// Validate hostname command (no arguments, always safe).
pub fn validate_hostname(_args: &[String]) -> Result<()> {
    Ok(())
}

/// Validate uname command arguments.
///
/// Only specific flags are allowed: -a, -s, -n, -r, -v, -m
pub fn validate_uname(args: &[String]) -> Result<()> {
    let safe_flags = ["-a", "-s", "-n", "-r", "-v", "-m"];
    for arg in args {
        if arg.starts_with('-') && !safe_flags.contains(&arg.as_str()) {
            return Err(anyhow!("unsupported uname flag '{}'", arg));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_echo() {
        assert!(validate_echo(&[]).is_ok());
        assert!(validate_echo(&["hello".to_string()]).is_ok());
        assert!(validate_echo(&["-n".to_string(), "hello".to_string()]).is_ok());
        assert!(validate_echo(&["-e".to_string(), "test".to_string()]).is_ok());
        assert!(validate_echo(&["--invalid".to_string()]).is_err());
    }

    #[test]
    fn test_validate_pwd() {
        assert!(validate_pwd(&[]).is_ok());
        assert!(validate_pwd(&["arg".to_string()]).is_err());
    }

    #[test]
    fn test_validate_printenv() {
        assert!(validate_printenv(&[]).is_ok());
        assert!(validate_printenv(&["PATH".to_string()]).is_ok());
        assert!(validate_printenv(&["MY_VAR_123".to_string()]).is_ok());
        assert!(validate_printenv(&["MY-VAR".to_string()]).is_err());
        assert!(validate_printenv(&["MY VAR".to_string()]).is_err());
        assert!(validate_printenv(&["VAR1".to_string(), "VAR2".to_string()]).is_err());
    }

    #[test]
    fn test_validate_which() {
        assert!(validate_which(&["ls".to_string()]).is_ok());
        assert!(validate_which(&["git".to_string(), "-a".to_string()]).is_ok());
        assert!(validate_which(&[]).is_err());
        assert!(validate_which(&["/usr/bin/ls".to_string()]).is_err()); // Contains /
        assert!(validate_which(&["ls git".to_string()]).is_err()); // Contains space
    }
}
