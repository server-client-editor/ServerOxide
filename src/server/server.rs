use crate::settings::Settings;

pub struct Server {

}

impl Server {
    pub fn try_new(settings: &Settings) -> anyhow::Result<Self> {
        Ok(Self { })
    }
}