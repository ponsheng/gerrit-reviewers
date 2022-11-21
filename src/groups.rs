// TODO Store queried reviewers under local directories
use rand::{distributions::Alphanumeric, Rng};
use log::trace;
use std::hash::{Hash, Hasher};
use linked_hash_set::LinkedHashSet;

//use crate::gerrit_if;
use crate::users::{GerritUser, UsersTy};
use crate::config;
use crate::config::{};

pub type UserGroups = LinkedHashSet<GerritUserGroup>;

#[derive(Eq, Clone)]
pub struct GerritUserGroup {
    pub users: UsersTy,
    pub name: String,
}

impl Hash for GerritUserGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for GerritUserGroup {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl IntoIterator for GerritUserGroup {
    type Item = GerritUser;
    type IntoIter = <LinkedHashSet<GerritUser> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.users.into_iter()
    }
}

/*
impl<'a> IntoIterator for &'a GerritUserGroup {
    type Item = &'a GerritUser;
    type IntoIter = std::vec::IntoIter<&'a GerritUser>;

    fn into_iter(self) -> Self::IntoIter {
        self.users.into_iter()
    }
}
*/


fn get_rand_string(char_num: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(char_num)
        .map(char::from)
        .collect()
}

pub fn gen_rand_groups() ->  UserGroups {
    let group_num = rand::random::<u8>() % 4 + 1;

    let mut groups = LinkedHashSet::new();
    for i in 0..group_num {
        let user_num = rand::random::<u8>() % 4 + 1;

        let mut users = LinkedHashSet::new();
        for i in 0..user_num {
            let username = get_rand_string(5);
            let u = GerritUser::from_string(username);
            users.insert(u);
        }

        let groupname = get_rand_string(2);
        let g = GerritUserGroup {
            users: users,
            name: groupname,
        };
        groups.insert(g);
    }

    groups
}

enum ConfigLevel {
    Local,
    Global,
}

pub fn write_config(gs: UserGroups) -> Result<String, String> {

    trace!("write_config");
    let mut cfg = config::GroupsConfig::new(config::ConfigScope::Local);
    cfg.clear();
    cfg.set(&gs);
    cfg.save();

    Ok(String::new())
}
