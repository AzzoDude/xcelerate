//! Codegen control: video recording and the `codegen` sub-commands.

use super::record::codegen_language;
use super::state::Session;

impl Session {
    /// `record [path]` / `rec`: start a video capture of the live run.
    pub(crate) async fn record(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Start a video capture of the live run. Tied to codegen: a
        // bare `--codegen` run records *actions*; `record` adds a
        // screen capture of the same run.
        if self.args.codegen.is_none() {
            println!("record is only available with --codegen <LANG>");
            return Ok(());
        }
        if self.video_path.is_some() {
            println!("already recording; `stop-record` first");
            return Ok(());
        }
        let output = if rest.is_empty() {
            "recording.mp4".to_string()
        } else {
            rest.to_string()
        };
        self.page.start_video(output.clone()).await?;
        self.video_path = Some(output);
        println!("recording started (will write on `stop-record`)");
        Ok(())
    }

    /// `stop-record` / `stop-rec` / `record-stop`: finalize the video.
    pub(crate) async fn stop_record(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.args.codegen.is_none() {
            println!("stop-record is only available with --codegen <LANG>");
            return Ok(());
        }
        match self.page.stop_video().await? {
            Some(path) => {
                println!("wrote {path}");
                self.video_path = None;
            }
            None => println!("no recording was in progress"),
        }
        Ok(())
    }

    /// `codegen [preview|out|undo|remove <n>|clear|status]` / `gen`.
    pub(crate) async fn codegen(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.args.codegen.is_none() {
            println!("codegen is only available with --codegen <LANG>");
            return Ok(());
        }
        let (sub, arg) = match rest.split_once(char::is_whitespace) {
            Some((sub, arg)) => (sub.to_ascii_lowercase(), arg.trim().to_string()),
            None => (rest.to_ascii_lowercase(), String::new()),
        };
        let lang = self.args.codegen.expect("checked above");
        match sub.as_str() {
            // `codegen preview` / `codegen`: print the script so far.
            "" | "preview" | "show" | "print" => {
                let code = codegen_language(lang).generate(&self.recording);
                println!(
                    "\n# ---- generated {} ({} actions, preview) ----\n{code}",
                    codegen_language(lang).label(),
                    self.recording.len()
                );
            }
            // `codegen out [path]`: write a snapshot without ending
            // the session (useful when the AI wants to hand off a draft).
            "out" | "write" | "save" => {
                let code = codegen_language(lang).generate(&self.recording);
                let path = if arg.is_empty() {
                    codegen_language(lang).file_name().to_string()
                } else {
                    arg
                };
                match std::fs::write(&path, code.as_bytes()) {
                    Ok(()) => println!("codegen: wrote {path} ({} actions)", self.recording.len()),
                    Err(error) => eprintln!("codegen: could not write {path}: {error}"),
                }
            }
            // `codegen undo` / `codegen remove <n>`: prune mistakes
            // from the recorded run before it is rendered.
            "undo" => {
                if self.recording.pop().is_some() {
                    println!(
                        "removed the last recorded action ({} left)",
                        self.recording.len()
                    );
                } else {
                    println!("nothing recorded yet");
                }
            }
            "remove" | "drop" => {
                let n = arg.parse::<usize>().unwrap_or(1);
                if self.recording.len() <= n {
                    self.recording.clear();
                } else {
                    self.recording.truncate(self.recording.len() - n);
                }
                println!("removed {n} action(s) ({} left)", self.recording.len());
            }
            // `codegen clear`: start the recording over.
            "clear" | "reset" => {
                self.recording.clear();
                self.last_target = None;
                println!("recording cleared");
            }
            "status" | "stats" | "count" => {
                println!("{} action(s) recorded", self.recording.len());
            }
            _ => {
                println!("usage: codegen [preview|out|undo|remove <n>|clear|status]");
            }
        }
        Ok(())
    }
}
