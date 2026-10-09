//! Network and environment verbs: request interception (`route`), HTTP auth
//! credentials, browser permissions, and geolocation override.
//!
//! These drive the core interception surface (`Page::route*`) plus two CDP calls
//! the core does not wrap (`Browser.grantPermissions`,
//! `Emulation.setGeolocationOverride`).

use xcelerate_interpreter::runtime::resolve_in_root;

use super::state::Session;

impl Session {
    /// `route <pattern>` / `route abort <pattern>` /
    /// `route fulfill <pattern> <status> <body> [content-type]` /
    /// `route har <path>`: register a request-interception rule.
    pub(crate) async fn route(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sub = tokens
            .get(1)
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        match sub.as_str() {
            // `route <pattern>`: intercept matching requests and let them through
            // (they are recorded, so `requests`/`request` can inspect them).
            "" => println!(
                "usage: route <pattern> | route abort <pattern> | \
                 route fulfill <pattern> <status> <body> [content-type] | route har <path>"
            ),
            "abort" => {
                let pattern = tokens.get(2).cloned().unwrap_or_default();
                if pattern.is_empty() {
                    println!("usage: route abort <pattern>");
                    return Ok(());
                }
                self.page.route_abort(pattern.clone()).await?;
                println!("route abort {pattern}");
            }
            "fulfill" => {
                let pattern = tokens.get(2).cloned().unwrap_or_default();
                let body = tokens.get(4).cloned();
                let Some(body) = body else {
                    println!("usage: route fulfill <pattern> <status> <body> [content-type]");
                    return Ok(());
                };
                if pattern.is_empty() {
                    println!("usage: route fulfill <pattern> <status> <body> [content-type]");
                    return Ok(());
                }
                let status = tokens
                    .get(3)
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(200);
                // The core interception pump answers every fulfill with 200; the
                // CLI accepts a status for symmetry but cannot change it.
                if status != 200 {
                    println!("note: fulfill is served with 200 regardless of {status}");
                }
                let content_type = tokens.get(5).cloned();
                self.page
                    .route_fulfill(pattern.clone(), body, content_type)
                    .await?;
                println!("route fulfill {pattern}");
            }
            "har" => {
                let raw = tokens.get(2).cloned().unwrap_or_default();
                if raw.is_empty() {
                    println!("usage: route har <path>");
                    return Ok(());
                }
                let path = resolve_in_root(&self.root, &raw)?;
                self.page
                    .route_from_har(path.to_string_lossy().into_owned())
                    .await?;
                println!("route har {}", path.display());
            }
            // The first token is the pattern: `route <pattern>`.
            pattern => {
                self.page
                    .route(pattern.to_string(), "continue".to_string(), None, None)
                    .await?;
                println!("route {pattern}");
            }
        }
        Ok(())
    }

    /// `unroute [pattern]`: drop the rules for `pattern`, or every rule.
    pub(crate) async fn unroute(&mut self, rest: &str) -> Result<(), Box<dyn std::error::Error>> {
        if rest.is_empty() {
            self.page.unroute_all().await?;
            println!("routes cleared");
        } else {
            self.page.unroute(rest.to_string()).await?;
            println!("route removed for {rest}");
        }
        Ok(())
    }

    /// `auth <username> <password>`: credentials for HTTP basic-auth challenges.
    pub(crate) async fn auth(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let username = tokens.get(1).cloned().unwrap_or_default();
        let password = tokens.get(2).cloned().unwrap_or_default();
        if username.is_empty() {
            println!("usage: auth <username> <password>");
            return Ok(());
        }
        self.page.authenticate(username.clone(), password).await?;
        println!("http auth credentials set for {username}");
        Ok(())
    }

    /// `permissions <origin> <permission> [permission...]`: grant browser
    /// permissions to an origin (`Browser.grantPermissions`).
    pub(crate) async fn permissions(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let origin = tokens.get(1).cloned().unwrap_or_default();
        let permissions: Vec<String> = tokens.iter().skip(2).cloned().collect();
        if origin.is_empty() || permissions.is_empty() {
            println!("usage: permissions <origin> <permission> [permission...]");
            return Ok(());
        }
        let json = serde_json::to_string(&permissions)?;
        self.browser.grant_permissions(origin.clone(), json).await?;
        println!("granted {} permission(s) to {origin}", permissions.len());
        Ok(())
    }

    /// `geolocation <latitude> <longitude> [accuracy]` / `geolocation clear`.
    ///
    /// The core exposes no `Page` geolocation helper, so this uses the CDP
    /// `Emulation.setGeolocationOverride` command directly via
    /// `Page::execute_cdp_cmd`.
    pub(crate) async fn geolocation(
        &mut self,
        tokens: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first = tokens
            .get(1)
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if first == "clear" {
            self.page
                .execute_cdp_cmd(
                    "Emulation.clearGeolocationOverride".to_string(),
                    "{}".to_string(),
                )
                .await?;
            println!("geolocation cleared");
            return Ok(());
        }
        let latitude = tokens.get(1).and_then(|s| s.parse::<f64>().ok());
        let longitude = tokens.get(2).and_then(|s| s.parse::<f64>().ok());
        match (latitude, longitude) {
            (Some(latitude), Some(longitude)) => {
                let accuracy = tokens
                    .get(3)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                self.page
                    .execute_cdp_cmd(
                        "Emulation.setGeolocationOverride".to_string(),
                        serde_json::json!({
                            "latitude": latitude,
                            "longitude": longitude,
                            "accuracy": accuracy
                        })
                        .to_string(),
                    )
                    .await?;
                println!("geolocation set to {latitude},{longitude} (+/-{accuracy}m)");
            }
            _ => {
                println!("usage: geolocation <latitude> <longitude> [accuracy] | geolocation clear")
            }
        }
        Ok(())
    }
}
