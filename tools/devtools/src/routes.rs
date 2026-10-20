#[derive(Clone, Debug, PartialEq, guinea::Routable)]
pub enum Route {
    Home {},
    Elements { app: u64 },
    Traces { app: u64 },
    Profiler { app: u64 },
    Application { app: u64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppParams {
    pub app: u64,
}

impl Route {
    pub fn app(&self) -> Option<u64> {
        match self {
            Self::Home {} => None,
            Self::Elements { app }
            | Self::Traces { app }
            | Self::Profiler { app }
            | Self::Application { app } => Some(*app),
        }
    }
}
