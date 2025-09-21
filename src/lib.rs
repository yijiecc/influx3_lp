//! A serializer for Rust structs according to [InfluxDB 3 line protocol](https://docs.influxdata.com/influxdb3/core/reference/line-protocol/).
//!
//! When writing data to InfluxDB 3 Core, we have to [write_lp API](https://docs.influxdata.com/influxdb3/core/write-data/http-api/v3-write-lp/) which differs from JSON.
//! This crate defines a derive macro called `Influx3Lp` to assit write process.
//! 
//! ```rust
//! use influx3_lp::Influx3Lp;
//!
//! #[derive(Influx3Lp)]
//! #[influx3_lp(table_name = "home")]
//! struct SensorData {
//!     pub temp: f32,
//!     pub hum: f64,
//!     pub co: Option<i32>,
//!     pub weather: String,
//!     #[influx3_lp(timestamp)]
//!     pub timestamp: i64,
//!     #[influx3_lp(tag)]
//!     pub room: String,
//!  }
//!    
//!  let data = SensorData {
//!      temp: 21.0,
//!      hum: 35.9,
//!      co: Some(0),
//!      weather: String::from("sunny"),
//!      timestamp: 1735545600,
//!      room: String::from("Kitchen"),
//!  };
//!
//!  // call `to_lp` function to transform struct to a String
//!  let serialized = data.to_lp();
//!
//!  assert_eq!(serialized, 
//!             "home,room=Kitchen temp=21,hum=35.9,co=0i,weather=\"sunny\" 1735545600")
//! ```
//!
//! These are features implemented by `influx3_lp`:
//! 
//! * `#[influx3_lp(timestamp)]` attribute is optional
//! * multiple `#[influx3_lp(tag)]` atrributes are supported
//! * empty tag is supported also
//! * tag values and field values are [escaped according to line protocol](https://docs.influxdata.com/influxdb3/core/reference/line-protocol/#special-characters)
//! * field string has a length limit of 64K
//! * `i8`,`i16`,`i32`,`i64` field values are appended with `i`
//! * `u8`,`u16`,`u32`,`u64` field values are appended with `u`
//! * field type of `Option<T>` is supported

////////////////////////////////////////////////////////////////////////////////

pub use influx3_lp_macros::*;

/// This is the trait that `Influx3Lp` macro help us implementing.
pub trait Influx3Lp {
    /// After decorating a struct with `#[derive(Influx3Lp)]` macro, we can call `to_lp` method directly to a line protocol string.
    ///
    /// Please pay attention: Influx table_name, tag keys and field keys are checked at compile time, but tag values and field values can only be checked at runtime. So please use valid tag/field values or panic will occur.
    fn to_lp(&self) -> String;
}

