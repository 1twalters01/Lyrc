use configuration::config::Config;

use crate::{
    provider::LyricsTranslator,
    providers::{
        argos::ArgosTranslator, deepl::DeepLTranslator,
        google::GoogleTranslator, huggingface::HuggingfaceTranslator,
        ollama::OllamaTranslator,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum TranslateService {
    Argos,
    DeepL,
    Google,
    Huggingface,
    Ollama,
}

impl TranslateService {
    pub fn as_str(&self) -> String {
        match self {
            Self::Argos => String::from("Argos"),
            Self::DeepL => String::from("DeepL"),
            Self::Google => String::from("Google"),
            Self::Huggingface => String::from("Huggingface"),
            Self::Ollama => String::from("Ollama"),
        }
    }

    pub fn get_provider(&self) -> Box<dyn LyricsTranslator> {
        match self {
            Self::Argos => {
                Box::new(ArgosTranslator) as Box<dyn LyricsTranslator>
            }
            Self::DeepL => {
                Box::new(DeepLTranslator) as Box<dyn LyricsTranslator>
            }
            Self::Google => {
                Box::new(GoogleTranslator) as Box<dyn LyricsTranslator>
            }
            Self::Huggingface => {
                Box::new(HuggingfaceTranslator) as Box<dyn LyricsTranslator>
            }
            Self::Ollama => {
                Box::new(OllamaTranslator) as Box<dyn LyricsTranslator>
            }
        }
    }

    pub fn get_prefered_provider(config: &Config) -> Self {
        Self::Argos
    }

    pub fn get_providers(config: &Config) -> Vec<TranslateService> {
        return Vec::from([
            Self::Argos,
            Self::DeepL,
            Self::Google,
            Self::Huggingface,
            Self::Ollama,
        ]);
    }
}
