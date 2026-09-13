use gleamler::{Env, NifTaggedEnum, OwnedEnv, Subject, gleam_nif, init_nifs};
use std::thread;
use std::time::Duration;

/// Messages sent from the background worker to the Gleam actor
#[derive(NifTaggedEnum, Debug)]
pub enum WorkerMsg {
    Progress(i64),
    Done(String),
}

/// Starts a long-running task in an OS background thread.
/// Returns immediately to Gleam without blocking the BEAM scheduler.
/// Automatically cancels the background thread if the calling Gleam actor dies.
#[gleam_nif]
fn start_work(env: Env, subject: Subject<WorkerMsg>, steps: i64) -> bool {
    let cancel_token = match env.cancellation_token() {
        Ok(token) => token,
        Err(_) => return false,
    };

    let thread_env = OwnedEnv::new();
    let saved_subject = subject.save(&thread_env);

    thread::spawn(move || {
        for step in 1..=steps {
            if cancel_token.is_cancelled() {
                // Gleam actor has terminated or cancelled; exit immediately
                return;
            }

            thread::sleep(Duration::from_millis(100));

            // Send progress update back to the Gleam process
            let _ = saved_subject.send(&thread_env, WorkerMsg::Progress(step));
        }

        if cancel_token.is_cancelled() {
            return;
        }

        // Send final completion message
        let _ = saved_subject.send(
            &thread_env,
            WorkerMsg::Done(format!("Successfully finished all {} steps!", steps)),
        );
    });

    true
}

init_nifs!();
