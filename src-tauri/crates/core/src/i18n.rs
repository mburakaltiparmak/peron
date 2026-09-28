//! Backend-side strings (tray, notifications, durations). UI strings live in `src/i18n.ts`.

use serde::{Deserialize, Serialize};

use crate::platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Tr,
    En,
}

impl Lang {
    fn parse(s: &str) -> Option<Lang> {
        match s.trim().to_ascii_lowercase().as_str() {
            "tr" => Some(Lang::Tr),
            "en" => Some(Lang::En),
            _ => None,
        }
    }

    /// Settings choice → installer choice (Windows setup asks) → OS display language → English.
    /// Language is asked during setup only; afterwards it's changed in Settings / tray menu.
    pub fn resolve(chosen: Option<Lang>) -> Lang {
        chosen
            .or_else(|| platform::installer_language().and_then(|s| Lang::parse(&s)))
            .unwrap_or(if platform::ui_language_is_turkish() {
                Lang::Tr
            } else {
                Lang::En
            })
    }
}

pub enum Text {
    OpenApp,
    Quit,
    Language,
    NoDevPorts,
    PickToClose,
    Tooltip(usize),
    ReminderTitle,
    ReminderBody {
        process: String,
        port: u16,
        duration: String,
    },
    ExposedTitle,
    ExposedBody {
        process: String,
        port: u16,
        addrs: String,
    },
}

pub fn t(lang: Lang, text: Text) -> String {
    use Lang::*;
    match (text, lang) {
        (Text::OpenApp, Tr) => "Peron'u aç".into(),
        (Text::OpenApp, En) => "Open Peron".into(),
        (Text::Quit, Tr) => "Çıkış".into(),
        (Text::Quit, En) => "Quit".into(),
        (Text::Language, Tr) => "Dil".into(),
        (Text::Language, En) => "Language".into(),
        (Text::NoDevPorts, Tr) => "Açık geliştirme portu yok".into(),
        (Text::NoDevPorts, En) => "No open dev ports".into(),
        (Text::PickToClose, Tr) => "Kapatmak için seçin:".into(),
        (Text::PickToClose, En) => "Select to close:".into(),
        (Text::Tooltip(n), Tr) => format!("Peron — {n} açık port"),
        (Text::Tooltip(n), En) => format!("Peron — {n} open port{}", if n == 1 { "" } else { "s" }),
        (Text::ReminderTitle, Tr) => "Açık port hatırlatması".into(),
        (Text::ReminderTitle, En) => "Open port reminder".into(),
        (
            Text::ReminderBody {
                process,
                port,
                duration,
            },
            Tr,
        ) => {
            format!("{process} {port} portunu {duration} süredir açık tutuyor.")
        }
        (
            Text::ReminderBody {
                process,
                port,
                duration,
            },
            En,
        ) => {
            format!("{process} has kept port {port} open for {duration}.")
        }
        (Text::ExposedTitle, Tr) => "Ağa açık yeni port".into(),
        (Text::ExposedTitle, En) => "New port open to the network".into(),
        (
            Text::ExposedBody {
                process,
                port,
                addrs,
            },
            Tr,
        ) => {
            format!("{process}, {port} portunu ağdaki diğer cihazlara açtı ({addrs}).")
        }
        (
            Text::ExposedBody {
                process,
                port,
                addrs,
            },
            En,
        ) => {
            format!("{process} opened port {port} to other devices on the network ({addrs}).")
        }
    }
}

/// "3 sa 12 dk" / "3 h 12 min" style duration.
pub fn duration(lang: Lang, ms: i64) -> String {
    let mins = (ms.max(0) / 60_000) as u64;
    let (d, h, m) = (mins / 1440, (mins % 1440) / 60, mins % 60);
    let (dl, hl, ml) = match lang {
        Lang::Tr => ("gün", "sa", "dk"),
        Lang::En => ("d", "h", "min"),
    };
    match (d, h) {
        (0, 0) => format!("{m} {ml}"),
        (0, _) => format!("{h} {hl} {m} {ml}"),
        _ => format!("{d} {dl} {h} {hl}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(duration(Lang::Tr, 30_000), "0 dk");
        assert_eq!(duration(Lang::Tr, (3 * 60 + 12) * 60_000), "3 sa 12 dk");
        assert_eq!(duration(Lang::Tr, 26 * 60 * 60_000), "1 gün 2 sa");
        assert_eq!(duration(Lang::En, (3 * 60 + 12) * 60_000), "3 h 12 min");
    }

    #[test]
    fn parse() {
        assert_eq!(Lang::parse("TR"), Some(Lang::Tr));
        assert_eq!(Lang::parse(" en "), Some(Lang::En));
        assert_eq!(Lang::parse("de"), None);
        assert_eq!(Lang::resolve(Some(Lang::En)), Lang::En);
    }
}
