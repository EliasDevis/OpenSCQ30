use std::borrow::Cow;

pub use equalizer::*;
use openscq30_i18n::Translate;
use openscq30_i18n_macros::Translate;
pub use range::*;
pub use select::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoEnumIterator, IntoStaticStr, VariantArray};
pub use value::*;

use crate::i18n::fl;

mod equalizer;
mod range;
mod select;
mod value;

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Hash,
    Clone,
    Serialize,
    Deserialize,
    Translate,
    Display,
    EnumString,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum CategoryId {
    General,
    SoundModes,
    Equalizer,
    ButtonConfiguration,
    DeviceInformation,
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Hash,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    EnumString,
    Translate,
    Display,
    VariantArray,
    IntoStaticStr,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
// Removing or renaming anything here will break quick presets, so this enum should be append only.
// If something really needs to be renamed, use #[strum(serialize = "...")] to keep the representation the same.
pub enum SettingId {
    AmbientSoundMode,
    TransparencyMode,
    NoiseCancelingMode,
    CustomNoiseCanceling,
    #[translate("preset-profile")]
    PresetEqualizerProfile,
    #[translate("custom-profile")]
    CustomEqualizerProfile,
    VolumeAdjustments,
    LeftSinglePress,
    LeftDoublePress,
    LeftLongPress,
    RightSinglePress,
    RightDoublePress,
    RightLongPress,
    NormalModeInCycle,
    TransparencyModeInCycle,
    NoiseCancelingModeInCycle,
    AdaptiveNoiseCanceling,
    ManualNoiseCanceling,
    WindNoiseSuppression,
    AdaptiveNoiseCancelingSensitivityLevel,
    IsCharging,
    BatteryLevel,
    IsChargingLeft,
    BatteryLevelLeft,
    IsChargingRight,
    BatteryLevelRight,
    SerialNumber,
    FirmwareVersion,
    FirmwareVersionLeft,
    FirmwareVersionRight,
    TwsStatus,
    HostDevice,
    StateUpdatePacket,
    MultiSceneNoiseCanceling,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Setting {
    Toggle {
        value: bool,
    },
    I32Range {
        setting: Range<i32>,
        value: i32,
    },
    // Select/OptionalSelect is just a hint about whether None is an acceptable value or not.
    // The backing data is still Option<u16> for both and should be treated the same by the backend.
    Select {
        setting: Select,
        value: Cow<'static, str>,
    },
    OptionalSelect {
        setting: Select,
        value: Option<Cow<'static, str>>,
    },
    /// Allows the user to add/remove items from the select
    ModifiableSelect {
        setting: Select,
        value: Option<Cow<'static, str>>,
    },
    Equalizer {
        setting: Equalizer,
        value: Vec<i16>,
    },
    Information {
        value: String,
        translated_value: String,
    },
}

impl From<Setting> for Value {
    fn from(setting: Setting) -> Self {
        match setting {
            Setting::Toggle { value, .. } => value.into(),
            Setting::I32Range { value, .. } => value.into(),
            Setting::Select { value, .. } => value.into(),
            Setting::OptionalSelect { value, .. } => value.into(),
            Setting::Equalizer { value, .. } => value.into(),
            Setting::ModifiableSelect { value, .. } => value.into(),
            Setting::Information {
                value: text,
                translated_value: _,
            } => Cow::<str>::Owned(text).into(),
        }
    }
}

impl Setting {
    pub(crate) fn select_from_enum_all_variants<T>(value: T) -> Self
    where
        T: PartialEq + Into<&'static str> + IntoEnumIterator + Translate,
    {
        Self::Select {
            setting: Select::from_enum(T::iter()),
            value: Cow::Borrowed(value.into()),
        }
    }

    pub(crate) fn optional_select_from_enum_all_variants<T>(value: Option<T>) -> Self
    where
        T: PartialEq + Into<&'static str> + IntoEnumIterator + Translate,
    {
        Setting::OptionalSelect {
            setting: Select::from_enum(T::iter()),
            value: value.map(|v| Cow::Borrowed(v.into())),
        }
    }

    pub(crate) fn select_from_enum<T>(variants: &[T], value: T) -> Self
    where
        for<'a> &'a T: PartialEq + Into<&'static str>,
        T: Into<&'static str> + Translate,
    {
        Self::Select {
            setting: Select::from_enum(variants),
            value: Cow::Borrowed(value.into()),
        }
    }
}

pub fn localize_value(setting: Option<&Setting>, value: &Value) -> String {
    match setting {
        Some(
            Setting::Select { setting, .. }
            | Setting::OptionalSelect { setting, .. }
            | Setting::ModifiableSelect { setting, .. },
        ) => match value.try_as_optional_str() {
            Ok(Some(selection)) => setting
                .options
                .iter()
                .position(|option| option == selection)
                .and_then(|index| setting.localized_options.get(index))
                .cloned()
                .unwrap_or_else(|| fl!("none")),
            Ok(None) => fl!("none"),
            Err(_) => value.to_string(),
        },
        _ => value.to_string(),
    }
}
