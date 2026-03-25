//! AssetLoader Bevy para archivos `.ron` → [`ElementDef`](super::element_def::ElementDef).

use bevy::asset::{AssetLoader, LoadContext, io::Reader};

use super::element_def::ElementDef;

/// Loader Bevy para `.ron` que mapea al tipo [`ElementDef`].
///
/// AssetServer + AssetEvent garantizan hot-reload; nosotros “swappeamos” el
/// `AlchemicalAlmanac` completo de forma atómica cuando cambia cualquier `.ron`.
#[derive(Default)]
pub struct ElementDefRonLoader;

impl AssetLoader for ElementDefRonLoader {
    type Asset = ElementDef;
    type Settings = ();
    type Error = ron::error::SpannedError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let def = ron::de::from_bytes::<ElementDef>(&bytes)?;
        Ok(def)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
