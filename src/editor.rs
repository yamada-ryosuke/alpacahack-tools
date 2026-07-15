use std::process;

use crate::prelude::*;

/// VSCodeで問題ディレクトリを開く。
pub fn open_with_vscode(project: &Project) -> Result<()> {
    process::Command::new("code")
        .arg(project.get_path())
        .spawn()?
        .wait()?;
    Ok(())
}