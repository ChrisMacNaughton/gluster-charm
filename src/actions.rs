use gluster;
use juju;
use log::LogLevel;

use std::path::PathBuf;
use std::str::FromStr;

pub fn enable_volume_quota() -> Result<(), String> {
    // Gather our action parameters
    let volume = match juju::action_get("volume") {
        Ok(v) => v,
        Err(e) => {
            // Notify the user of the failure and then return the error up the stack
            juju::action_fail(&e.to_string()).map_err(|e| e.to_string())?;
            return Err(e.to_string());
        }
    };
    let usage_limit = match juju::action_get("usage-limit") {
        Ok(usage) => usage,
        Err(e) => {
            // Notify the user of the failure and then return the error up the stack
            juju::action_fail(&e.to_string()).map_err(|e| e.to_string())?;
            return Err(e.to_string());
        }
    };
    let parsed_usage_limit = u64::from_str(&usage_limit).map_err(|e| e.to_string())?;
    let path = match juju::action_get("path") {
        Ok(p) => p,
        Err(e) => {
            // Notify the user of the failure and then return the error up the stack
            juju::action_fail(&e.to_string()).map_err(|e| e.to_string())?;
            return Err(e.to_string());
        }
    };

    // Turn quotas on if not already enabled
    let quotas_enabled = gluster::volume_quotas_enabled(&volume).map_err(|e| e.to_string())?;
    if !quotas_enabled {
        gluster::volume_enable_quotas(&volume).map_err(|e| e.to_string())?;
    }

    gluster::volume_add_quota(&volume, PathBuf::from(path), parsed_usage_limit)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn disable_volume_quota() -> Result<(), String> {
    // Gather our action parameters
    let volume = match juju::action_get("volume") {
        Ok(v) => v,
        Err(e) => {
            // Notify the user of the failure and then return the error up the stack
            juju::action_fail(&e.to_string()).map_err(|e| e.to_string())?;
            return Err(e.to_string());
        }
    };
    let path = match juju::action_get("path") {
        Ok(p) => p,
        Err(e) => {
            // Notify the user of the failure and then return the error up the stack
            juju::action_fail(&e.to_string()).map_err(|e| e.to_string())?;
            return Err(e.to_string());
        }
    };

    let quotas_enabled = gluster::volume_quotas_enabled(&volume).map_err(|e| e.to_string())?;
    if quotas_enabled {
        match gluster::volume_remove_quota(&volume, PathBuf::from(path)) {
            Ok(_) => return Ok(()),
            Err(e) => {
                // Notify the user of the failure and then return the error up the stack
                juju::action_fail(&e.to_string()).map_err(|e| e.to_string())?;
                return Err(e.to_string());
            }
        }
    } else {
        return Ok(());
    }
}

pub fn list_volume_quotas() -> Result<(), String> {
    // Gather our action parameters
    let volume = match juju::action_get("volume") {
        Ok(v) => v,
        Err(e) => {
            // Notify the user of the failure and then return the error up the stack
            juju::log(&format!("Failed to get volume param: {:?}", e),
                      Some(LogLevel::Debug));
            juju::action_fail(&e.to_string()).map_err(|e| e.to_string())?;
            return Err(e.to_string());
        }
    };
    let quotas_enabled = gluster::volume_quotas_enabled(&volume).map_err(|e| e.to_string())?;
    if quotas_enabled {
        match gluster::quota_list(&volume) {
            Ok(quotas) => {
                let quota_string: Vec<String> = quotas.iter()
                    .map(|quota| {
                        format!("path: {:?} limit: {} used: {}",
                                quota.path,
                                quota.limit,
                                quota.used)
                    })
                    .collect();
                juju::action_set("quotas", &quota_string.join("\n")).map_err(|e| e.to_string())?;
                return Ok(());
            }
            Err(e) => {
                juju::log(&format!("Quota list failed: {:?}", e),
                          Some(LogLevel::Error));
                return Err(e.to_string());
            }
        }
    } else {
        juju::log(&format!("Quotas are disabled on volume: {}", volume),
                  Some(LogLevel::Debug));
        return Ok(());
    }
}

pub fn set_volume_options() -> Result<(), String> {
    // volume is a required parameter so this should be safe
    let mut volume: String = String::new();

    // Gather all of the action parameters up at once.  We don't know what
    // the user wants to change.
    let options = juju::action_get_all().map_err(|e| e.to_string())?;
    let mut settings: Vec<gluster::GlusterOption> = Vec::new();
    for (key, value) in options {
        if key != "volume" {
            settings.push(try!(gluster::GlusterOption::from_str(&key, value)
                .map_err(|e| e.to_string())));
        } else {
            volume = value;
        }
    }
    gluster::volume_set_options(&volume, settings).map_err(|e| e.to_string())?;
    return Ok(());
}
