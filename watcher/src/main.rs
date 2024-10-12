use std::{
    path::Path,
    process::{Command, Stdio},
};

use notify::{Event, RecursiveMode, Result, Watcher};

fn main() -> Result<()> {
    let mut watcher = notify::recommended_watcher(|res| match res {
        Ok(event) => handle_event(event),
        Err(e) => println!("watch error: {:?}", e),
    })?;

    loop {
        watcher.watch(Path::new("../recordings/"), RecursiveMode::Recursive)?;
    }
}

fn handle_event(event: Event) {
    match event.kind {
        notify::EventKind::Create(_) => {
            for path in event.paths {
                copy_file(&path);
            }
        }

        _ => {}
    }
}

fn copy_file(path: &Path) {
    // wait 200 ms for the file to be fully written
    std::thread::sleep(std::time::Duration::from_millis(1000));

    let _status = Command::new("cp")
        .arg(path)
        .arg("../".to_string())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to run scp");
}
