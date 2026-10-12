use std::{
    path::Path,
    process::{Command, Stdio},
};

use notify::{Event, RecursiveMode, Result, Watcher};

fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("no path given");
    let mut watcher = notify::recommended_watcher(|res| match res {
        Ok(event) => handle_event(event),
        Err(e) => println!("watch error: {:?}", e),
    })?;

    watcher.watch(Path::new(&path), RecursiveMode::Recursive)?;

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
    std::thread::sleep(std::time::Duration::from_millis(500));

    let destination = "christian@91.98.26.93:/home/christian/other_apps/satellites/recordings";
    let _status = Command::new("scp")
        .arg(path)
        .arg(destination)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("failed to run scp");
}
