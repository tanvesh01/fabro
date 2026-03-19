use anyhow::Result;
use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default)
        .interact_on(&Term::stderr())?)
}

pub fn input(prompt: &str) -> Result<String> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_on(&Term::stderr())?
        .trim()
        .to_string())
}

pub fn input_with_default(prompt: &str, default: &str) -> Result<String> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default.to_string())
        .interact_on(&Term::stderr())?
        .trim()
        .to_string())
}

pub fn positive_i64(prompt: &str, default: i64) -> Result<i64> {
    loop {
        let value = Input::<i64>::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .interact_on(&Term::stderr())?;
        if value > 0 {
            return Ok(value);
        }
        eprintln!("{prompt} must be greater than 0.");
    }
}

pub fn select(prompt: &str, items: &[String], default: usize) -> Result<usize> {
    Ok(Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(default)
        .interact_on(&Term::stderr())?)
}
