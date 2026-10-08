//! Session-control verbs (help, quit, done) and the family of waits.

use std::sync::Arc;
use std::time::Duration;

use super::dispatch::Control;
use super::help::print_session_help;
use super::state::Session;

/// Prints the command list; a step that never touches the page.
pub(crate) fn help() {
    print_session_help();
}

impl Session {
    /// `quit`: leave the loop. In an AI-driven run this is a last resort: the job
    /// must be marked done first (`done`), otherwise the agent is urged to finish
    /// or to use `quit!`.
    pub(crate) fn quit(&mut self) -> Result<Control, Box<dyn std::error::Error>> {
        // An AI-driven run is not allowed to walk away from the browser
        // mid-task. Exiting is a last resort: the job must be declared done
        // (`done`), or the agent must override it explicitly (`quit!`) when it
        // has genuinely hit a wall - missing information or a feature gap it
        // should surface to the user instead.
        if self.args.ai && !self.job_done {
            println!(
                "not exiting: the job is not marked done.\n\
                 \x20 finish the task and run `done`, or if you truly cannot\n\
                 \x20 proceed (missing info / a needed feature), run `quit!` and\n\
                 \x20 tell the user what is blocking you."
            );
            return Ok(Control::Continue);
        }
        Ok(Control::Quit)
    }

    /// `quit!` / `exit!` / `q!`: the explicit last-resort override.
    pub(crate) fn quit_force(&mut self) -> Result<Control, Box<dyn std::error::Error>> {
        // Explicit last-resort override: the agent acknowledges it is leaving
        // before the job is complete.
        if self.args.ai && !self.job_done {
            println!(
                "exiting before the job is done - please explain to the user\n\
                 \x20 what is missing or blocked."
            );
        }
        Ok(Control::Quit)
    }

    /// `done` / `complete`: mark the task complete so a later `quit` is allowed.
    pub(crate) fn done(&mut self, rest: &str) {
        if rest.is_empty() {
            println!("usage: done [short summary]   (marks the task complete)");
        } else {
            println!("marked done: {rest}");
        }
        self.job_done = true;
    }

    /// `wait <ms|selector>` / `sleep`: sleep, or wait for an element.
    pub(crate) async fn wait_for(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ms) = xcelerate_interpreter::runtime::parse_duration_ms(rest) {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            println!("waited {ms}ms");
        } else if rest.is_empty() {
            println!("usage: wait <ms|selector>");
        } else {
            let started = std::time::Instant::now();
            match Arc::clone(&self.page)
                .wait_for_selector(rest.to_string())
                .await
            {
                Ok(_) => {
                    println!("found {rest} after {}ms", started.elapsed().as_millis());
                }
                Err(_) => {
                    println!("timed out waiting for {rest}");
                    self.step_no_op = true;
                }
            }
        }
        Ok(())
    }

    /// `wait-stable [ms]` / `stable`: wait until the DOM stops changing.
    pub(crate) async fn wait_stable(
        &mut self,
        rest: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Wait only as long as the page keeps changing, then stop -
        // the productive wait for a page that renders progressively.
        let quiet = rest.parse::<u64>().unwrap_or(700);
        match self.page.wait_for_dom_stable(quiet, 60_000).await {
            Ok(()) => println!("dom settled ({quiet}ms quiet)"),
            Err(error) => println!("not settled: {error}"),
        }
        Ok(())
    }

    /// `wait-idle [ms]` / `idle`: wait until the network goes quiet.
    pub(crate) async fn wait_idle(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        let quiet = rest.parse::<u64>().unwrap_or(500);
        match self.page.wait_for_network_idle(quiet, 60_000).await {
            Ok(()) => println!("network idle ({quiet}ms quiet)"),
            Err(error) => println!("not idle: {error}"),
        }
        Ok(())
    }
}

/// A unit-suffixed sleep: `wait-ms` / `wait-sec` / `wait-min` / `wait-hr`. The
/// unit is in the verb and the value is a plain number (there is no `2s`
/// literal).
pub(crate) async fn wait_unit(rest: &str, unit_ms: u64, name: &str) {
    match xcelerate_interpreter::runtime::wait_scaled(rest, unit_ms).await {
        Ok(msg) => println!("{msg}"),
        Err(e) => println!("usage: {name} <number>  ({e})"),
    }
}

/// `wait-random <min> <max>`: sleep a uniform random hold, in milliseconds.
pub(crate) async fn wait_random(rest: &str) {
    let mut bounds = rest.split_whitespace();
    match (
        bounds.next().map(xcelerate_interpreter::runtime::parse_ms),
        bounds.next().map(xcelerate_interpreter::runtime::parse_ms),
    ) {
        (Some(Ok(min)), Some(Ok(max))) => {
            let ms = xcelerate_interpreter::runtime::random_ms(min, max);
            tokio::time::sleep(Duration::from_millis(ms as u64)).await;
            println!("waited {ms}ms");
        }
        _ => println!("usage: wait-random <min> <max>   (milliseconds)"),
    }
}
