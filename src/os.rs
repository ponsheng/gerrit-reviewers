use std::str;
use log::{trace};
use std::process::Command;

pub fn run_command_exc(cmd_vec : Vec<&str>) -> Result<String, String> {

    trace!("Running: {}", cmd_vec.join(" "));

    let cmd = cmd_vec[0];
    let result = Command::new(cmd)
        .args(&cmd_vec[1..])
        .output()
        .expect("failed to execute process");

    if !result.status.success() {
        let err_msg = str::from_utf8(&result.stderr).expect("Invalid UTF8-8 sequence");
        return Err(err_msg.to_string())
    }

    let out = str::from_utf8(&result.stdout).expect("Invalid UTF8-8 sequence");
    // TODO json pretty print
    trace!("{}", out);
    Ok(out.to_string())
}

