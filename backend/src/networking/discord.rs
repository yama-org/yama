use anyhow::{bail, Ok};
use discord_sdk as ds;
use std::ops::Deref;
use std::time::{Duration, SystemTime};
use std::{fmt, sync::Arc};
use tracing::{error, info};

pub const APP_ID: ds::AppId = 1128545330837856316;

#[derive(Debug)]
pub struct Discord {
    pub ds: DiscordWrapper,
    pub user: ds::user::User,
    pub wheel: WheelWrapper,
}

impl Discord {
    pub async fn new(subs: ds::Subscriptions) -> crate::Result<Discord> {
        let (wheel, handler) = ds::wheel::Wheel::new(Box::new(|err| {
            error!(error = ?err, "encountered an error");
        }));

        let mut user = wheel.user();
        let ds = ds::Discord::new(ds::DiscordApp::PlainId(APP_ID), subs, Box::new(handler))?;

        info!("Waiting for handshake...");
        if let Err(e) =
            tokio::time::timeout(std::time::Duration::from_secs(5), user.0.changed()).await
        {
            error!("Could no connect to discord: {e}");
            bail!(ds::Error::NoConnection)
        }

        let user = match &*user.0.borrow() {
            ds::wheel::UserState::Connected(user) => user.clone(),
            ds::wheel::UserState::Disconnected(err) => {
                error!("Failed to connect to Discord: {}", err);
                bail!(ds::Error::NoConnection);
            }
        };

        info!("Connected to Discord, local user is {}", user.username);

        Ok(Discord {
            ds: DiscordWrapper(ds),
            user,
            wheel: WheelWrapper(wheel),
        })
    }

    pub async fn cleanup(self) -> crate::Result<()> {
        self.ds.clear_activity().await?;
        self.ds.0.disconnect().await;
        Ok(())
    }

    pub async fn idle_activity(&self) {
        let rp = ds::activity::ActivityBuilder::default()
            .details("Picking anime to watch...".to_string())
            .state("Idle".to_string())
            .assets(
                ds::activity::Assets::default()
                    .large("megane".to_string(), Some("b-baka!!".to_string())),
                //.small("megane".to_string(), Some("...".to_string())),
            )
            .button(ds::activity::Button {
                label: "yama by sad-ko".to_string(),
                url: "https://github.com/yama-org/yama".to_string(),
            });

        info!(
            "Update discord activity: {:?}",
            self.ds.update_activity(rp).await
        );
    }

    pub async fn watch_activity(
        &self,
        title_name: Arc<str>,
        episode_name: Arc<str>,
        remaining_time: f64,
    ) {
        let duration = Duration::from_secs_f64(remaining_time);

        let rp = ds::activity::ActivityBuilder::default()
            .details(title_name.deref())
            .state("Watching".to_string())
            .assets(
                ds::activity::Assets::default()
                    .large("megane".to_string(), Some(episode_name.deref())),
            )
            .button(ds::activity::Button {
                label: "yama by sad-ko".to_string(),
                url: "https://github.com/yama-org/yama".to_string(),
            });

        info!(
            "Update discord activity: {:?}",
            self.ds
                .update_activity(match SystemTime::now().checked_add(duration) {
                    Some(remaining_time) => rp.end_timestamp(remaining_time),
                    None => rp,
                })
                .await
        );
    }
}

pub struct DiscordWrapper(ds::Discord);

impl fmt::Debug for DiscordWrapper {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("Discord Wrapper")
    }
}

impl std::ops::Deref for DiscordWrapper {
    type Target = ds::Discord;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct WheelWrapper(ds::wheel::Wheel);

impl fmt::Debug for WheelWrapper {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("Discord Wheel Wrapper")
    }
}

impl std::ops::Deref for WheelWrapper {
    type Target = ds::wheel::Wheel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
