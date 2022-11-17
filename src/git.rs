
use regex::Regex;

use crate::os;
use log::{info, debug, warn};
use crate::gerrit_if;
use crate::gerrit_if::GerritUser;

fn git_config_get_value(section : &str, option : &str) -> Result<String, String> {
    let name = format!("{}.{}", section, option);
    let cmd = vec!["git", "config", "--get", &*name];

    let result = os::run_command_exc(cmd);

    result
}


// FIXME
fn alias_url(url : String, _rewrite_push : bool) -> String {
    url
}

// FIXME remote not respected
pub fn get_remote_url(_remote : String) -> String {
    let push_url;
    let ret = git_config_get_value("remote.origin", "pushurl");
    match ret {
        Ok(s) => push_url = alias_url(s, false),

        Err(_) => {
            let ret = git_config_get_value("remote.origin", "url");
            match ret {
                Ok(s) => push_url = alias_url(s, true),
                Err(_) => panic!("Failed to get remote url")
            }
        }
    }
    debug!("Found origin Push URL: {}", push_url);
    push_url
}


fn get_git_directories() -> (String, String) {
    let ret = os::run_command_exc(vec!["git", "rev-parse", "--show-toplevel", "--git-dir"]);
    match ret {
        Ok(s) => {
            let mut lines = s.split('\n');
            let top_dir = lines.nth(0).expect("Vec out of bound").to_string();
            let git_dir = lines.nth(1).expect("Vec out of bound").to_string();
            (top_dir, git_dir)
        },
        Err(_) => panic!("Cannot find .git directory")
    }
}

// Return "" if command failed
fn get_local_commit_message(git_ref: &str) -> String {
    let cmd = vec!["git", "show", "-s", "--format=medium", git_ref];
    match os::run_command_exc(cmd) {
        Ok(stdout) => stdout,
        Err(stderr) => "".to_string(),
    }
}

/*
// Max: 8
fn get_unmerged_local_changes() -> Vec<String> {
    info!("Searching for target commit");

    let mut commits = Vec::new();

    for backward_index in 0..8 {
        let git_ref = format!("HEAD~{}", backward_index);
        info!("Ref: {}", git_ref);
        match get_local_commit_change_id(&git_ref) {
            Some(id) => {
                if id.len() == 0 {
                    continue;
                }
                info!("    Change id:{}", id);
                if gerrit_if::is_change_open(&id) {
                    commits.push(id);
                }
            },
            None => break,
        }
    }
    commits
}
*/

// Return None if failed to use git_ref
// Return "" if ChangeID not found
fn get_local_commit_change_id(git_ref: &str) -> Option<String> {
    let re = Regex::new(r"^\s*Change-Id: ([A-Za-z0-9]+)$").unwrap();

    let message = get_local_commit_message(git_ref);

    if message.len() == 0 {
        return None;
    }

    let mut ret = "".to_string();
    for line in message.split('\n') {
        match re.captures(line) {
            Some(caps) => {
                let id = caps.get(1).unwrap().as_str();
                if ret.len() > 0 {
                    warn!("Multiple Change-Id in commit");
                    continue;
                }
                ret = id.to_string();
            },
            None => continue,
        }
    }
    Some(ret)
}

