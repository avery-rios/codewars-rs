use serde::{de, Deserialize, Deserializer};

fn deserialize_opt_str<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Option<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(if v.is_empty() {
                None
            } else {
                Some(v.to_string())
            })
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(if v.is_empty() { None } else { Some(v) })
        }
    }
    deserializer.deserialize_str(Visitor)
}

pub type OptString = Option<String>;
pub type Time = u32;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "t")]
pub enum Output {
    Describe {
        #[serde(rename = "p")]
        pass: bool,
        v: String,
        #[serde(default)]
        items: Vec<Output>,
    },
    It {
        #[serde(rename = "p")]
        pass: bool,
        v: String,
        #[serde(default)]
        items: Vec<Output>,
    },
    Passed {
        v: String,
    },
    Failed {
        v: String,
    },
    Log {
        v: String,
    },
    Error {
        v: String,
    },
    CompletedIn {
        v: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct RunStat {
    pub passed: u32,
    pub failed: u32,
}

#[derive(Debug, Deserialize)]
pub struct RunStatHidden {
    pub passed: u32,
    pub failed: u32,
    pub hidden: RunStat,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub server_error: bool,
    pub completed: bool,
    pub output: Vec<Output>,
    // pub success_mode: (),
    pub passed: u32,
    pub failed: u32,
    pub errors: u32,
    // pub error: Option<()>,
    pub assertions: RunStatHidden,
    pub specs: RunStatHidden,
    pub unweighted: RunStat,
    pub weighted: RunStat,
    pub timed_out: bool,
    pub wall_time: Time,
    pub test_time: Option<Time>,
    // pub tags: Option<()>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub exit_code: u32,
    pub token: String,
    #[serde(deserialize_with = "deserialize_opt_str")]
    pub message: OptString,
    #[serde(deserialize_with = "deserialize_opt_str")]
    pub stdout: OptString,
    #[serde(deserialize_with = "deserialize_opt_str")]
    pub stderr: OptString,
    pub result: Box<RunResult>, // reduce struct size
}
