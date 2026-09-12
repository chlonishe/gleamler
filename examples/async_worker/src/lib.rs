use gleamler::{NifTaggedEnum, OwnedEnv, Subject, gleam_nif, init_nifs};
use std::thread;
use std::time::Duration;

/// Messages sent from the background worker to the Gleam actor
#[derive(NifTaggedEnum, Debug)]
pub enum WorkerMsg {
    Progress(i64),
    Done(String),
}

/// Starts a long-running task in an OS background thread.
/// Returns immediately to Gleam without blocking the BEAM scheduler
#[gleam_nif]
fn start_work(subject: Subject<WorkerMsg>, steps: i64) -> bool {
    let pid = subject.pid();
    let tag = subject.tag();

    // Create a process-independent environment for the background thread
    let mut thread_env = OwnedEnv::new();
    let saved_tag = thread_env.save(tag);

    thread::spawn(move || {
        for step in 1..=steps {
            thread::sleep(Duration::from_millis(100));

            // Send progress update back to the Gleam process
            let _ = Subject::send_from_owned(
                &pid,
                &saved_tag,
                &mut thread_env,
                WorkerMsg::Progress(step),
            );
        }

        // Send final completion message
        let _ = Subject::send_from_owned(
            &pid,
            &saved_tag,
            &mut thread_env,
            WorkerMsg::Done(format!("Successfully finished all {} steps!", steps)),
        );
    });

    true
}

init_nifs!();
