use std::fs::File;
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use log::trace;
use linked_hash_set::LinkedHashSet;
use regex::Regex;
use dirs::home_dir;

use crate::groups::{UserGroups, GerritUserGroup};
use crate::users::GerritUser;
use crate::users;
use crate::git;

const CONFIG_NAME: &'static str = "gerreviewer.ini";

pub enum ConfigScope {
    Local,
    Global,
}

pub struct GroupsConfig {
    scope: ConfigScope,
    pub file_path: String,
    pub groups: UserGroups,
}

pub struct RecentUsersConfig;

impl GroupsConfig {
    // TODO, new, get, save, set, clear
    pub fn new(scope: ConfigScope) -> Self {

        let file_path = get_config_path(&scope);

        let mut config = Self {
            scope: scope,
            file_path: file_path.to_string(),
            groups: LinkedHashSet::new(),
        };

        // Load 
        if Path::new(&mut config.file_path).exists() {
            let mut contents = String::new();
            let mut file = File::open(config.file_path.clone()).unwrap();
            
            file.read_to_string(&mut contents).unwrap();
            config.load(contents);
            trace!("Loaded {}", config.file_path);
        } else {
            trace!("{} not found", config.file_path);
        }

        
        config
    }

    fn load(&mut self, contents: String) {
        trace!("Load config");
        let group_name_matcher = Regex::new(r"\[([[:word:]]+)\]").unwrap();

        let lines = contents.lines();
        let mut has_cur_group = false;
        let mut cur_group = GerritUserGroup {
            name: String::new(),
            users: LinkedHashSet::new(),
        };
        'lines: for line in lines {
            match group_name_matcher.captures(line) {
                Some(caps) => {
                    let group_name = caps.get(1).unwrap().as_str();
                    cur_group.name = group_name.to_string();
                    has_cur_group = true;
                    continue;
                },
                None => (),
            };

            if !has_cur_group {
                continue;
            }

            // parse user list
            let user_strs = line.split(",");
            if user_strs.clone().count() == 0 {
                continue;
            }

            cur_group.users.clear();
            'users: for _username in user_strs {
                let username = _username.trim();
                if !users::is_valid_username(username) {
                    continue 'users;
                }
                cur_group.users.insert(GerritUser::from_str(username));
            }
            trace!("Parsed group: {}", cur_group.name);
            self.groups.insert(cur_group.clone());
            has_cur_group = false;
        }
    }

    pub fn clear(&mut self) {
        self.groups.clear();
    }

    pub fn set(&mut self, groups: &UserGroups) {
        trace!("set");
        for group in groups {
            self.groups.insert(group.clone());
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let mut s = String::new();
        for g in &self.groups {
            s.push_str(&self.to_config_string(&g));
            s.push_str("\n\n");
        }
        let mut file = File::create(self.file_path.clone())?;
        file.write_all(s.as_bytes())?;
        trace!("Saved {}", self.file_path);
        Ok(())
    }

    fn to_config_string(&self, g: &GerritUserGroup) -> String {
        let mut ret = format!("[{}]\n", &g.name);
        ret.push_str("  ");
        let num = g.users.len();
        for (pos, u) in g.users.iter().enumerate() {
            // TODO check if contains space
            ret.push_str(&u.username);
            if pos != (num - 1) {
                ret.push_str(", ");
            }
        }
        return ret;
    }
}

fn get_config_path(scope: &ConfigScope) -> String {
    let dir_pathbuf = match scope {
        ConfigScope::Local => {
            let git_dir = git::get_git_directories().unwrap();
            PathBuf::from(git_dir)
        }
        ConfigScope::Global => {
            let home = home_dir().unwrap().into_os_string().to_str().unwrap().to_string();
            PathBuf::from(home).join(".config").join("gerreviewer")
        }
    };
    let path = dir_pathbuf.join(CONFIG_NAME);
    path.to_str().unwrap().to_string()
}

pub fn get_group_configs() -> Vec<GroupsConfig> {
    let local = get_local_groups();
    let global = get_global_groups();

    vec![local, global]
}

// FIXME use Box?
fn get_local_groups() -> GroupsConfig {
    GroupsConfig::new(ConfigScope::Local)
}

fn get_global_groups() -> GroupsConfig {
    GroupsConfig::new(ConfigScope::Global)
}
