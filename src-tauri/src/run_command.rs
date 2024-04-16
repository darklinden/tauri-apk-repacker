use anyhow::Result;

pub async fn run_command(program: &str, args: &[&str]) -> Result<String> {
    let full_command = format!("{} {}", program, args.join(" "));
    log::info!("running command: {}", full_command);

    let output = std::process::Command::new(program).args(args).output()?;
    if output.status.code() != Some(0) {
        if !output.stderr.is_empty() {
            log::error!("error: {}", String::from_utf8_lossy(&output.stderr));
        }
        return Err(anyhow::anyhow!("run_command failed"));
    }

    let out_str = String::from_utf8_lossy(&output.stdout);
    log::info!("output: {}", out_str);

    Ok(out_str.to_string())
}
