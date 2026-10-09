//! Verb routing: the one `match` that maps a lexed command onto its group.
//!
//! The arm bodies live in the sibling modules, grouped by concern; this file only
//! owns the routing table and preserves every verb alias exactly.

use super::misc;
use super::state::Session;

/// Whether the session keeps running after a dispatched step.
pub(crate) enum Control {
    /// Keep reading commands.
    Continue,
    /// Leave the loop (`quit` / `quit!`).
    Quit,
}

impl Session {
    /// Routes a lexed session command to the group that implements it.
    ///
    /// `verb` is the lowercased first token, `tokens` the full lexed line, and
    /// `rest` the remaining tokens rejoined with single spaces.
    pub(crate) async fn dispatch(
        &mut self,
        verb: &str,
        tokens: &[String],
        rest: &str,
    ) -> Result<Control, Box<dyn std::error::Error>> {
        match verb {
            // Session control.
            "help" | "?" => misc::help(),
            "quit" => return self.quit(),
            "quit!" | "exit!" | "q!" => return self.quit_force(),
            "done" | "complete" => self.done(rest),

            // Navigation and reading.
            "open" | "goto" => self.open(rest).await?,
            "back" => self.back().await?,
            "reload" => self.reload().await?,
            "title" => self.title().await?,
            "url" => self.url().await?,
            "text" => self.text().await?,
            "markdown" | "md" => self.markdown().await?,
            "content" | "html" => self.content().await?,
            "snapshot" | "snap" => self.snapshot().await?,
            "find" => self.find(rest).await?,
            "scroll" => self.scroll(rest).await?,
            "eval" | "js" => self.eval(rest).await?,
            "challenge" | "detect" => self.challenge().await?,
            "await-human" | "await" | "human" => self.await_human(rest).await?,
            "guard" => self.guard(rest).await?,

            // Interaction.
            "click" => self.click(rest).await?,
            "tap" => self.tap(rest).await?,
            "drag" => self.drag(rest).await?,
            "dialog" => self.dialog(rest).await?,
            "fill" => self.fill(tokens).await?,
            "select" => self.select(tokens).await?,
            "type" => self.type_text(rest).await?,
            "press" | "submit" | "send" => self.press(verb, rest).await?,
            "hover" => self.hover(rest).await?,
            "mouse" => self.mouse(tokens, rest).await?,
            "click-xy" | "click-at" | "xy" => self.click_xy(rest).await?,

            // Media and files.
            "shot" | "screenshot" => self.shot(rest, false).await?,
            "shot-full" | "screenshot-full" => self.shot(rest, true).await?,
            "media" => self.media().await?,
            "download" => self.download(tokens).await?,
            "upload" | "set-input-files" => self.upload(tokens).await?,

            // Cookies.
            "cookie" | "cookies" => self.cookie(tokens).await?,

            // Network interception, auth, permissions and environment.
            "route" => self.route(tokens).await?,
            "unroute" => self.unroute(rest).await?,
            "auth" => self.auth(tokens).await?,
            "permissions" => self.permissions(tokens).await?,
            "geolocation" => self.geolocation(tokens).await?,

            // Storage.
            "storage" => self.storage(tokens).await?,

            // Tabs.
            "new-tab" | "newtab" | "tab-new" => self.new_tab(rest).await?,
            "switch" | "tab" | "use" => self.switch(rest).await?,
            "tabs" => self.tabs().await?,
            "close-tab" | "closetab" => self.close_tab(rest).await?,

            // Codegen.
            "record" | "rec" => self.record(rest).await?,
            "stop-record" | "stop-rec" | "record-stop" => self.stop_record().await?,
            "codegen" | "gen" => self.codegen(rest).await?,

            // Waits (the unit is in the verb; the value is a plain number).
            "wait-ms" => misc::wait_unit(rest, 1, "wait-ms").await,
            "wait-sec" => misc::wait_unit(rest, 1_000, "wait-sec").await,
            "wait-min" => misc::wait_unit(rest, 60_000, "wait-min").await,
            "wait-hr" => misc::wait_unit(rest, 3_600_000, "wait-hr").await,
            "wait-random" => misc::wait_random(rest).await,
            "wait" | "sleep" => self.wait_for(rest).await?,
            "wait-stable" | "stable" => self.wait_stable(rest).await?,
            "wait-idle" | "idle" => self.wait_idle(rest).await?,

            other => println!("unknown command: {other}   (try `help`)"),
        }
        Ok(Control::Continue)
    }
}
