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

    watcher.watch(Path::new("../recordings/"), RecursiveMode::Recursive)?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
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

    let destination = "christian@satellites.local:/home/christian/satellites/recordings";
    let _status = Command::new("scp")
        .arg(path)
        .arg(destination)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to run scp");
}
