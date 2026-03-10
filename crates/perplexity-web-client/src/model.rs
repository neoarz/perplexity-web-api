use std::fmt;
use std::str::FromStr;

pub(crate) const DEEP_RESEARCH_PREFERENCE: &str = "pplx_alpha";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPreference(&'static str);

impl ModelPreference {
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

macro_rules! define_models {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$var_meta:meta])*
                $variant:ident => { name: $model_name:literal, preference: $pref:literal }
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $($(#[$var_meta])* $variant,)+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            pub const VALID_NAMES: &'static [&'static str] = &[$($model_name),+];

            pub const fn as_str(&self) -> &'static str {
                match self { $(Self::$variant => $model_name,)+ }
            }

            pub const fn preference(&self) -> ModelPreference {
                match self { $(Self::$variant => ModelPreference($pref),)+ }
            }

            pub fn valid_names_csv() -> String {
                Self::VALID_NAMES.join(", ")
            }
        }

        impl From<$name> for ModelPreference {
            fn from(m: $name) -> Self {
                m.preference()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($model_name => Ok(Self::$variant),)+
                    _ => Err(format!(
                        "Unknown model '{s}'. Try one of: {}",
                        Self::valid_names_csv()
                    )),
                }
            }
        }
    };
}

define_models! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SearchModel {
        Turbo => { name: "turbo", preference: "turbo" },
        Sonar => { name: "sonar", preference: "experimental" },
        SonarPro => { name: "sonar-pro", preference: "pplx_pro" },
        Gemini30Flash => { name: "gemini-3-flash", preference: "gemini30flash" },
        Gpt52 => { name: "gpt-5.2", preference: "gpt52" },
        Claude46Sonnet => { name: "claude-4.6-sonnet", preference: "claude46sonnet" },
        Grok41 => { name: "grok-4.1", preference: "grok41nonreasoning" },
    }
}

define_models! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ReasonModel {
        SonarReasoning => { name: "sonar-reasoning", preference: "pplx_reasoning" },
        Gemini30FlashThinking => { name: "gemini-3-flash-thinking", preference: "gemini30flash_high" },
        Gemini31Pro => { name: "gemini-3.1-pro", preference: "gemini31pro_high" },
        Gpt52Thinking => { name: "gpt-5.2-thinking", preference: "gpt52_thinking" },
        Claude46SonnetThinking => { name: "claude-4.6-sonnet-thinking", preference: "claude46sonnetthinking" },
        Grok41Reasoning => { name: "grok-4.1-reasoning", preference: "grok41reasoning" },
        KimiK25Thinking => { name: "kimi-k2.5-thinking", preference: "kimik25thinking" },
    }
}
