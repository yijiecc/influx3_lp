use influx3_lp::Influx3Lp;

#[derive(Influx3Lp)]
#[influx3_lp(table_name = "home")]
struct SensorData {
}    

