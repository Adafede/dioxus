// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the dioxus-apps project

use super::*;

#[derive(Debug, Deserialize)]
pub(super) struct ConvertFormatsResponse {
    pub(super) canonical_smiles: String,
    pub(super) isomeric_smiles: String,
    pub(super) inchi: String,
    pub(super) inchikey: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize)]
struct RdkitConvertResponse {
    canonicalsmiles: String,
    isomericsmiles: String,
    inchi: String,
    inchikey: String,
}

pub(super) async fn convert_smiles(smiles: &str) -> Result<ConvertFormatsResponse, CurationError> {
    let canonicalsmiles = convert_with_batch(smiles, "canonicalsmiles").await?;
    let isomeric_smiles = convert_with_batch(smiles, "isomericsmiles").await?;
    let inchi = convert_with_batch(smiles, "inchi").await?;
    let inchikey = convert_with_batch(smiles, "inchikey").await?;
    Ok(ConvertFormatsResponse {
        canonical_smiles: canonicalsmiles,
        isomeric_smiles,
        inchi,
        inchikey,
    })
}

async fn convert_with_batch(smiles: &str, output_format: &str) -> Result<String, CurationError> {
    #[cfg(target_arch = "wasm32")]
    {
        return convert_with_rdkit(smiles, output_format).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        return convert_with_batch_direct(smiles, output_format).await;
    }
}

#[cfg(target_arch = "wasm32")]
async fn convert_with_rdkit(smiles: &str, output_format: &str) -> Result<String, CurationError> {
    let value = rdkit_bridge_call("convert", smiles.trim()).await?;
    let parsed = js_value_to_json(value)?;
    let converted = serde_json::from_value::<RdkitConvertResponse>(parsed)
        .map_err(|e| CurationError::Parse(format!("rdkit.js convert parse error: {e}")))?;
    match output_format {
        "canonicalsmiles" => Ok(converted.canonicalsmiles),
        "isomericsmiles" => Ok(converted.isomericsmiles),
        "inchi" => Ok(converted.inchi),
        "inchikey" => Ok(converted.inchikey),
        _ => Err(CurationError::InvalidInput(format!(
            "unsupported conversion format: {output_format}"
        ))),
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn convert_with_batch_direct(
    smiles: &str,
    output_format: &str,
) -> Result<String, CurationError> {
    let url = format!("{NATPROD_API_BASE}/convert/batch?output_format={output_format}");
    let payload = serde_json::json!({
        "inputs": [{ "value": smiles.trim(), "input_format": "smiles" }]
    });
    let response = natprod_client()
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            CurationError::Http(format!("naturalproducts convert/batch request error: {e}"))
        })?;
    if !response.status().is_success() {
        return Err(CurationError::Http(format!(
            "naturalproducts convert/batch failed with HTTP {}",
            response.status().as_u16()
        )));
    }
    let parsed = response.json::<BatchConvertResponse>().await.map_err(|e| {
        CurationError::Parse(format!("naturalproducts convert/batch parse error: {e}"))
    })?;
    extract_batch_convert_output(parsed)
}

#[cfg(not(target_arch = "wasm32"))]
fn extract_batch_convert_output(parsed: BatchConvertResponse) -> Result<String, CurationError> {
    let Some(first) = parsed.results.first() else {
        return Err(CurationError::Parse(
            "naturalproducts convert/batch returned no result rows".to_string(),
        ));
    };
    if !first.success {
        return Err(CurationError::InvalidInput(format!(
            "naturalproducts conversion failed: {}",
            first.error
        )));
    }
    Ok(first.output.clone())
}

pub(super) async fn descriptor_mass(smiles: &str) -> Result<f64, CurationError> {
    #[cfg(target_arch = "wasm32")]
    {
        return descriptor_mass_via_rdkit(smiles).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        return descriptor_mass_direct(smiles).await;
    }
}

#[cfg(target_arch = "wasm32")]
async fn descriptor_mass_via_rdkit(smiles: &str) -> Result<f64, CurationError> {
    let value = rdkit_bridge_call("exactMass", smiles.trim()).await?;
    value.as_f64().ok_or_else(|| {
        CurationError::Parse("rdkit.js exactMass did not return a number".to_string())
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn descriptor_mass_direct(smiles: &str) -> Result<f64, CurationError> {
    let url = format!(
        "{NATPROD_API_BASE}/chem/descriptors/multiple?smiles={}",
        urlencoding::encode(smiles.trim())
    );
    let response = natprod_client()
        .get(url)
        .send()
        .await
        .map_err(|e| CurationError::Http(e.to_string()))?;
    if !response.status().is_success() {
        return Err(CurationError::Http(format!(
            "naturalproducts.net chem/descriptors failed with HTTP {}",
            response.status().as_u16()
        )));
    }
    let parsed = response
        .json::<Value>()
        .await
        .map_err(|e| CurationError::Parse(e.to_string()))?;
    extract_exact_mass_from_json(&parsed)
        .ok_or_else(|| CurationError::Parse("missing exact_molecular_weight".to_string()))
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) fn extract_exact_mass_from_json(value: &Value) -> Option<f64> {
    if let Some(v) = value
        .get("exact_molecular_weight")
        .and_then(parse_exact_mass_scalar)
    {
        return Some(v);
    }
    if let Some(obj) = value.as_object() {
        for nested in obj.values() {
            if let Some(v) = extract_exact_mass_from_json(nested) {
                return Some(v);
            }
        }
    }
    if let Some(arr) = value.as_array() {
        for nested in arr {
            if let Some(v) = extract_exact_mass_from_json(nested) {
                return Some(v);
            }
        }
    }
    None
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn parse_exact_mass_scalar(value: &Value) -> Option<f64> {
    if let Some(v) = value.as_f64() {
        return Some(v);
    }
    if let Some(v) = value.as_i64() {
        return Some(v as f64);
    }
    if let Some(v) = value.as_u64() {
        return Some(v as f64);
    }
    value
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.replace(',', "").parse::<f64>().ok())
}

pub(super) async fn resolve_exact_mass(
    input_smiles: &str,
    canonical_smiles: &str,
) -> MassResolution {
    match descriptor_mass(canonical_smiles).await {
        Ok(value) => MassResolution {
            exact_mass: Some(value),
            warning: None,
        },
        Err(canonical_err) => {
            if canonical_smiles.trim() != input_smiles.trim() {
                match descriptor_mass(input_smiles).await {
                    Ok(value) => {
                        return MassResolution {
                            exact_mass: Some(value),
                            warning: None,
                        };
                    }
                    Err(input_err) => {
                        return MassResolution {
                            exact_mass: None,
                            warning: Some(format!(
                                "exact mass unavailable (canonical lookup failed: \
                                 {canonical_err}; input lookup failed: {input_err})"
                            )),
                        };
                    }
                }
            }
            MassResolution {
                exact_mass: None,
                warning: Some(format!("exact mass unavailable ({canonical_err})")),
            }
        }
    }
}

pub(super) async fn has_undefined_stereo(smiles: &str) -> bool {
    if has_stereo_marks(smiles) {
        return false;
    }

    #[cfg(target_arch = "wasm32")]
    {
        return rdkit_bridge_call("hasUndefinedStereo", smiles.trim())
            .await
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let url = format!(
            "{NATPROD_API_BASE}/chem/stereoisomers?smiles={}",
            urlencoding::encode(smiles.trim())
        );
        let Ok(response) = natprod_client().get(url).send().await else {
            return false;
        };
        let Ok(json) = response.json::<Value>().await else {
            return false;
        };
        json.get("stereoisomers")
            .and_then(Value::as_array)
            .map(|a| a.len() > 1)
            .unwrap_or(false)
    }
}
