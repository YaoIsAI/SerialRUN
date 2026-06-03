use crate::state::{PlcBrand, PlcDataType, PlcRegisterDef};

pub struct PlcModel {
    #[allow(dead_code)]
    pub brand: PlcBrand,
    pub model: &'static str,
    pub registers: Vec<PlcRegisterDef>,
}

pub fn get_models(brand: PlcBrand) -> Vec<PlcModel> {
    match brand {
        PlcBrand::Siemens => vec![siemens_s71200()],
        PlcBrand::Mitsubishi => vec![mitsubishi_fx3u()],
        PlcBrand::Delta => vec![delta_dvp()],
        PlcBrand::Omron => vec![omron_cp1h()],
        PlcBrand::Custom => vec![],
    }
}

fn r(addr: u16, name: &str, dt: PlcDataType, scale: f64, unit: &str, desc: &str) -> PlcRegisterDef {
    PlcRegisterDef { addr, name: name.into(), data_type: dt, scale_factor: scale, unit: unit.into(), description: desc.into() }
}

fn siemens_s71200() -> PlcModel {
    PlcModel {
        brand: PlcBrand::Siemens, model: "S7-1200",
        registers: vec![
            r(0, "Temperature SP", PlcDataType::Float32, 0.1, "\u{00b0}C", "Temperature setpoint"),
            r(2, "Temperature PV", PlcDataType::Float32, 0.1, "\u{00b0}C", "Temperature process value"),
            r(4, "Pressure", PlcDataType::Float32, 0.01, "bar", "System pressure"),
            r(8, "Speed SP", PlcDataType::U16, 1.0, "rpm", "Motor speed setpoint"),
            r(9, "Speed PV", PlcDataType::U16, 1.0, "rpm", "Motor speed actual"),
            r(10, "Motor Status", PlcDataType::U16, 1.0, "", "Motor status bits"),
            r(11, "Alarm Code", PlcDataType::U16, 1.0, "", "Alarm code"),
        ],
    }
}

fn mitsubishi_fx3u() -> PlcModel {
    PlcModel {
        brand: PlcBrand::Mitsubishi, model: "FX3U",
        registers: vec![
            r(0, "D0 - General", PlcDataType::I16, 1.0, "", "General purpose D0"),
            r(1, "D1 - General", PlcDataType::I16, 1.0, "", "General purpose D1"),
            r(4, "D4 - Counter", PlcDataType::U16, 1.0, "", "Counter value"),
            r(5, "D5 - Timer", PlcDataType::U16, 0.01, "s", "Timer value"),
            r(10, "Speed", PlcDataType::U16, 1.0, "rpm", "Motor speed"),
        ],
    }
}

fn delta_dvp() -> PlcModel {
    PlcModel {
        brand: PlcBrand::Delta, model: "DVP",
        registers: vec![
            r(0, "D0", PlcDataType::I16, 1.0, "", "Data register D0"),
            r(4, "Temperature", PlcDataType::U16, 0.1, "\u{00b0}C", "Temperature reading"),
            r(5, "Pressure", PlcDataType::U16, 0.01, "MPa", "Pressure reading"),
        ],
    }
}

fn omron_cp1h() -> PlcModel {
    PlcModel {
        brand: PlcBrand::Omron, model: "CP1H",
        registers: vec![
            r(0, "D0", PlcDataType::U16, 1.0, "", "DM area D0"),
            r(4, "Temperature", PlcDataType::U16, 0.1, "\u{00b0}C", "Temperature input"),
            r(5, "Setpoint", PlcDataType::U16, 0.1, "\u{00b0}C", "Temperature setpoint"),
            r(6, "Output", PlcDataType::U16, 0.1, "%", "Control output"),
        ],
    }
}
