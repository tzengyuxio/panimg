use crate::error::{PanimgError, Result};
use image::DynamicImage;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PsdInfo {
    pub width: u32,
    pub height: u32,
    pub layers: Vec<LayerInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayerInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

fn parse_psd(data: &[u8]) -> Result<psd::Psd> {
    psd::Psd::from_bytes(data).map_err(|e| PanimgError::DecodeError {
        message: format!("{e}"),
        path: None,
        suggestion: "check that the PSD file is valid".into(),
    })
}

fn layer_to_info(layer: &psd::PsdLayer, index: usize) -> LayerInfo {
    LayerInfo {
        index,
        name: layer.name().to_string(),
        width: u32::from(layer.width()),
        height: u32::from(layer.height()),
    }
}

fn decode_layer(layer: &psd::PsdLayer, index: usize) -> Result<DynamicImage> {
    let info = layer_to_info(layer, index);
    let rgba = layer.rgba();
    image::RgbaImage::from_raw(info.width, info.height, rgba)
        .map(DynamicImage::ImageRgba8)
        .ok_or_else(|| PanimgError::DecodeError {
            message: format!("failed to create image from layer {index}"),
            path: None,
            suggestion: "the layer data may be invalid".into(),
        })
}

pub fn get_psd_info(data: &[u8]) -> Result<PsdInfo> {
    let psd_file = parse_psd(data)?;

    let layers: Vec<LayerInfo> = psd_file
        .layers()
        .iter()
        .enumerate()
        .map(|(i, layer)| layer_to_info(layer, i))
        .collect();

    Ok(PsdInfo {
        width: psd_file.width(),
        height: psd_file.height(),
        layers,
    })
}

/// Process each layer through a callback, decoding one at a time to avoid
/// holding all decoded images in memory simultaneously.
/// The callback receives (LayerInfo, DynamicImage) and returns Ok(true) to
/// continue or Ok(false) to stop early.
pub fn for_each_layer<F>(data: &[u8], mut f: F) -> Result<usize>
where
    F: FnMut(LayerInfo, DynamicImage) -> Result<bool>,
{
    let psd_file = parse_psd(data)?;
    let total = psd_file.layers().len();

    for (i, layer) in psd_file.layers().iter().enumerate() {
        let info = layer_to_info(layer, i);
        let img = decode_layer(layer, i)?;
        if !f(info, img)? {
            break;
        }
    }

    Ok(total)
}
