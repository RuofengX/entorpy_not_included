use core::f32;
use std::{collections::HashMap, sync::LazyLock};

use serde_derive::{Deserialize, Serialize};

type TypeDict = HashMap<&'static str, &'static CellTy>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "phase")]
#[serde(rename_all = "lowercase")]
pub enum Phase {
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
pub struct CellTy {
    pub name: String,
    pub comment: String,
    #[serde(flatten)]
    pub phase: Phase,
}
impl CellTy {
    pub const ALL: LazyLock<TypeDict> = LazyLock::new(Self::load);
    fn load() -> TypeDict {
        let mut ret = HashMap::default();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_path("./res/cells.csv")
            .unwrap();

        for data in rdr.deserialize() {
            // deserialize
            eprintln!("{:?}", data);
            let ty: CellTy = data.unwrap();

            // leak to static
            let ty: &'static CellTy = Box::leak(Box::new(ty));
            let name: &'static str = Box::leak(ty.name.clone().into_boxed_str());

            // store
            ret.insert(name, ty);
        }
        ret
    }
    pub fn get_unchecked(name: &str) -> &'static CellTy {
        Self::ALL.get(name).unwrap()
    }
    pub fn check_transition(&'static self, temp: f32) -> Option<&'static CellTy> {
        match &self.phase {
            Phase::Gas {
                cold_temp,
                cold_product,
                ..
            } => {
                if temp < *cold_temp {
                    return Some(CellTy::get_unchecked(&cold_product));
                }
            }
            Phase::Liquid {
                hot_temp,
                hot_product,
                cold_temp,
                cold_product,
            } => {
                if temp < *cold_temp {
                    return Some(CellTy::get_unchecked(&cold_product));
                }
                if temp > *hot_temp {
                    return Some(CellTy::get_unchecked(&hot_product));
                }
            }
            Phase::Solid {
                hot_temp,
                hot_product,
            } => {
                if temp > *hot_temp {
                    return Some(CellTy::get_unchecked(&hot_product));
                }
            }
        }
        None
    }

    pub fn gas_pressure(&self, mass: f32, temperature: f32, volumn: f32) -> Option<f32> {
        if let Phase::Gas { molar_mass, .. } = self.phase {
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
        for i in CellTy::ALL.values() {
            println!("{:?}", i);
        }
    }
}
