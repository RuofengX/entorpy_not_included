use core::f32;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use serde_derive::{Deserialize, Serialize};

type TypeDict = HashMap<&'static str, Material>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "phase")]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Void,
    Gas {
        molar_mass: f32,
        cold_temp: f32,
        cold_product: String,
    },
    Solid {
        hot_temp: f32,
        hot_product: String,
    },
    Liquid {
        hot_temp: f32,
        hot_product: String,
        cold_temp: f32,
        cold_product: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaterialData {
    pub name: String,
    pub comment: String,
    #[serde(flatten)]
    pub phase: Phase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    inner: Arc<MaterialData>,
}

impl Material {
    pub const ALL: LazyLock<TypeDict> = LazyLock::new(Self::load);
    fn load() -> TypeDict {
        let mut ret = HashMap::default();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(include_str!("material.csv").as_bytes());

        for data in rdr.deserialize() {
            // deserialize
            let type_data: MaterialData = data.unwrap();

            // leak name to static
            let name: &'static str = Box::leak(type_data.name.clone().into_boxed_str());

            // build cell_ty
            let ty = Material {
                inner: Arc::new(type_data),
            };

            // store
            ret.insert(name, ty);
        }
        ret
    }

    #[inline]
    pub fn get_unchecked(name: &str) -> Material {
        Self::ALL.get(name).unwrap().clone()
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn comment(&self) -> &str {
        &self.inner.comment
    }

    pub fn phase(&self) -> &Phase {
        &self.inner.phase
    }

    /// 计算物态变化，如果变化，返回变化后的类型
    pub fn check_transition(&self, temp: f32) -> Option<Material> {
        match &self.inner.phase {
            Phase::Void => (),
            Phase::Gas {
                cold_temp,
                cold_product,
                ..
            } => {
                if temp < *cold_temp {
                    return Some(Material::get_unchecked(&cold_product));
                }
            }
            Phase::Liquid {
                hot_temp,
                hot_product,
                cold_temp,
                cold_product,
            } => {
                if temp < *cold_temp {
                    return Some(Material::get_unchecked(&cold_product));
                }
                if temp > *hot_temp {
                    return Some(Material::get_unchecked(&hot_product));
                }
            }
            Phase::Solid {
                hot_temp,
                hot_product,
            } => {
                if temp > *hot_temp {
                    return Some(Material::get_unchecked(&hot_product));
                }
            }
        }
        None
    }

    pub fn gas_pressure(&self, mass: f32, temperature: f32, volumn: f32) -> Option<f32> {
        if let Phase::Gas { molar_mass, .. } = self.inner.phase {
            // 将摄氏度转换为开尔文
            let temperature_kelvin = temperature + 273.15;
            // 理想气体常数 R = 8.314 J/(mol·K)
            let gas_constant = 8.314;
            // 计算摩尔数
            let moles = mass / molar_mass;
            // 计算压力
            let pressure = (moles * gas_constant * temperature_kelvin) / volumn;
            Some(pressure)
        } else {
            None
        }
    }
}
mod test {
    #[test]
    fn test_read_csv() {
        use super::*;
        for i in Material::ALL.values() {
            println!("{:?}", i);
        }
    }
}
